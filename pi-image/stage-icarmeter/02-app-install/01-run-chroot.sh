#!/bin/bash -e
# ==============================================================================
# Chroot-side: Verify binary, configure caps & enable autostart
# ==============================================================================

if [ ! -f /opt/icarmeter/car_dashboard ]; then
    echo "ERROR: /opt/icarmeter/car_dashboard was not installed into rootfs!" >&2
    exit 1
fi

# Set binary permissions and Bluetooth raw socket capabilities
chmod 755 /opt/icarmeter/car_dashboard
setcap 'cap_net_raw,cap_net_admin+eip' /opt/icarmeter/car_dashboard 2>/dev/null || true

# Setup convenience symlinks in PATH
ln -sf /opt/icarmeter/car_dashboard /usr/local/bin/car_dashboard
ln -sf /opt/icarmeter/car_dashboard /usr/local/bin/icarmeter

# Ensure SSH host keys are generated and present so ssh.service never fails
ssh-keygen -A || true

# Configure Bluetooth: Controller powered on for TPMS LE scanner (non-pairable, non-discoverable for phones)
mkdir -p /etc/bluetooth
cat << 'EOF' > /etc/bluetooth/main.conf
[General]
Name = ICarMeter
AutoEnable = true
ControllerMode = dual

[Policy]
AutoEnable = true
EOF

# Set system machine-info pretty hostname
echo 'PRETTY_HOSTNAME="ICarMeter"' > /etc/machine-info

# Ensure scripts and NetworkManager hotspot connection have proper permissions
chmod 755 /opt/icarmeter/icarmeter-bt-init.sh 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-wifi-init.sh 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-api.py 2>/dev/null || true
chmod 600 /etc/NetworkManager/system-connections/icarmeter-hotspot.nmconnection 2>/dev/null || true

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

# Configure NetworkManager: disable background wifi scanning (it can
# momentarily take wlan0 out of AP mode) and lock wlan0 to AP-only.
mkdir -p /etc/NetworkManager/conf.d
cat << 'EOF' > /etc/NetworkManager/conf.d/10-icarmeter-hotspot.conf
[device]
# Prevent NM from auto-scanning for station-mode wifi networks on wlan0.
# wlan0 is dedicated to the ICarMeter AP hotspot.
wifi.scan-rand-mac-address=no

[connection-wifi-wlan0-no-autoconnect]
# Block all station-mode wifi auto-connections. Only the ICarMeter-Hotspot
# AP profile (installed in system-connections/) is allowed.
match-device=interface-name:wlan0
connection.autoconnect=false

[main]
# Use internal DHCP client (not dhclient) for reliability
dhcp=internal
EOF

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
systemctl enable icarmeter-hotspot.service || true
systemctl enable icarmeter-api.service || true
