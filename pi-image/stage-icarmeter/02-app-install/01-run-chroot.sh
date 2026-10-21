#!/bin/bash -e
# ==============================================================================
# Chroot-side: Verify binary, configure caps & enable autostart
# ==============================================================================

if [ ! -f /opt/icarmeter/icarmeter ]; then
    echo "ERROR: /opt/icarmeter/icarmeter was not installed into rootfs!" >&2
    exit 1
fi

# Set binary permissions and Bluetooth raw socket capabilities
chmod 755 /opt/icarmeter/icarmeter
setcap 'cap_net_raw,cap_net_admin+eip' /opt/icarmeter/icarmeter 2>/dev/null || true

# Setup convenience symlink in PATH
ln -sf /opt/icarmeter/icarmeter /usr/local/bin/icarmeter

# Verify dynamic library linkage and execution
echo "=== Verifying dynamic libraries for /opt/icarmeter/icarmeter ==="
ldd /opt/icarmeter/icarmeter || true
if ldd /opt/icarmeter/icarmeter 2>&1 | grep -i "not found"; then
    echo "ERROR: /opt/icarmeter/icarmeter has missing shared libraries in rootfs!" >&2
    exit 1
fi

echo "=== Verifying /opt/icarmeter/icarmeter binary execution ==="
/opt/icarmeter/icarmeter --version

# Ensure SSH host keys are generated and present so ssh.service never fails
ssh-keygen -A || true

# Configure Bluetooth: Dual mode for TPMS LE scanner + RFCOMM SPP companion app
mkdir -p /etc/bluetooth
cat << 'EOF' > /etc/bluetooth/main.conf
[General]
Name = ICarMeter
Class = 0x000100
DiscoverableTimeout = 0
PairableTimeout = 0
AlwaysPairable = true
AutoEnable = true
ControllerMode = dual
JustWorksRepairing = always

[Policy]
AutoEnable = true
EOF

# BlueZ Compat Mode Drop-in: enables SDP server socket for SPP profile and grants permissions
BT_DAEMON="/usr/libexec/bluetooth/bluetoothd"
if [ ! -f "$BT_DAEMON" ] && [ -f "/usr/lib/bluetooth/bluetoothd" ]; then
    BT_DAEMON="/usr/lib/bluetooth/bluetoothd"
fi
mkdir -p /etc/systemd/system/bluetooth.service.d
cat << EOF > /etc/systemd/system/bluetooth.service.d/icarmeter-compat.conf
[Service]
ExecStart=
ExecStart=${BT_DAEMON} --compat
ExecStartPost=-/bin/chmod 777 /var/run/sdp
EOF

# Set system machine-info pretty hostname
echo 'PRETTY_HOSTNAME="ICarMeter"' > /etc/machine-info

# Hardware Wi-Fi stability: disable power saving and roaming in Broadcom driver
mkdir -p /etc/modprobe.d
cat << 'EOF' > /etc/modprobe.d/brcmfmac.conf
options brcmfmac roamoff=1 feature_disable=0x82000
EOF

# Udev rule: immediately turn off power management on wlan interfaces
mkdir -p /etc/udev/rules.d
cat << 'EOF' > /etc/udev/rules.d/80-wifi-powersave.rules
ACTION=="add", SUBSYSTEM=="net", KERNEL=="wlan*", RUN+="/sbin/iw dev %k set power_save off"
EOF

# Ensure scripts and NetworkManager hotspot connection have proper permissions
chmod 755 /opt/icarmeter/icarmeter-bt-init.sh 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-wifi-init.sh 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-api.py 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-bt-agent.py 2>/dev/null || true
chmod 666 /opt/icarmeter/config.toml 2>/dev/null || true
mkdir -p /home/${FIRST_USER_NAME}/.config/icarmeter 2>/dev/null || true
chmod 777 /home/${FIRST_USER_NAME}/.config/icarmeter 2>/dev/null || true
chmod 666 /home/${FIRST_USER_NAME}/.config/icarmeter/config.toml 2>/dev/null || true
chmod 600 /etc/NetworkManager/system-connections/icarmeter-hotspot.nmconnection 2>/dev/null || true
touch /var/log/icarmeter-bluetooth.log && chmod 666 /var/log/icarmeter-bluetooth.log 2>/dev/null || true

# Ensure Wi-Fi is enabled in NetworkManager state
mkdir -p /var/lib/NetworkManager
cat << 'EOF' > /var/lib/NetworkManager/NetworkManager.state
[main]
NetworkingEnabled=true
WirelessEnabled=true
WWANEnabled=true
EOF

# Set Wi-Fi country code to unblock wireless bands
raspi-config nonint do_wifi_country VN 2>/dev/null || true

# Offline mode optimizations: do not block or wait for external internet on boot
systemctl disable systemd-networkd-wait-online.service 2>/dev/null || true
systemctl disable NetworkManager-wait-online.service 2>/dev/null || true

# Prevent the system-level dnsmasq.service from conflicting with
# NetworkManager's own dnsmasq instance (method=shared spawns its own).
systemctl disable dnsmasq.service 2>/dev/null || true
systemctl mask dnsmasq.service 2>/dev/null || true

# Configure NetworkManager: disable background wifi scanning and force powersave off
mkdir -p /etc/NetworkManager/conf.d
cat << 'EOF' > /etc/NetworkManager/conf.d/10-icarmeter-hotspot.conf
[device]
# Prevent NM from auto-scanning for station-mode wifi networks on wlan0.
wifi.scan-rand-mac-address=no

[connection]
wifi.powersave=2

[main]
# Use internal DHCP client for reliability
dhcp=internal
EOF

# Do not let a dispatcher callback race the hotspot watchdog during radio
# bring-up or recovery. The hotspot service is the single activation owner.
rm -f /etc/NetworkManager/dispatcher.d/99-icarmeter-hotspot

# Boot flow: Plymouth -> tty1 autologin -> /etc/profile.d dashboard launcher.
# Keep getty enabled; it also restarts the dashboard session after a crash.
systemctl unmask getty@tty1.service 2>/dev/null || true
mkdir -p /etc/systemd/system/getty@tty1.service.d
cat << EOF > /etc/systemd/system/getty@tty1.service.d/icarmeter-autologin.conf
[Unit]
After=plymouth-quit-wait.service

[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin ${FIRST_USER_NAME} --noclear %I \$TERM
EOF
systemctl daemon-reload || true
systemctl enable getty@tty1.service || true

# Enable autostart services on boot
systemctl daemon-reload || true
systemctl enable bluetooth.service 2>/dev/null || true
systemctl enable hciuart.service 2>/dev/null || true
systemctl enable icarmeter-bluetooth.service || true
systemctl enable icarmeter-bt-agent.service || true
systemctl enable icarmeter-hotspot.service || true
systemctl enable icarmeter-api.service || true
