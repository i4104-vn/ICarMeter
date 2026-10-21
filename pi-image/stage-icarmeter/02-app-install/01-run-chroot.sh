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
Class = 0x200404
DiscoverableTimeout = 0
PairableTimeout = 0
AlwaysPairable = true
AutoEnable = true
ControllerMode = dual
FastConnectable = true
Privacy = off
JustWorksRepairing = always

[Policy]
AutoEnable = true
ReconnectAttempts = 7
ReconnectIntervals = 1, 2, 4, 8, 16, 32, 64
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
ExecStartPost=-/bin/chmod 777 /var/run/sdp /run/sdp
EOF

# Set system machine-info pretty hostname
echo 'PRETTY_HOSTNAME="ICarMeter"' > /etc/machine-info

# Ensure Bluetooth scripts and configuration have proper permissions
chmod 755 /opt/icarmeter/icarmeter-bt-init.sh 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-api.py 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-bt-agent.py 2>/dev/null || true
chmod 666 /opt/icarmeter/config.toml 2>/dev/null || true
mkdir -p /home/${FIRST_USER_NAME}/.config/icarmeter 2>/dev/null || true
chmod 777 /home/${FIRST_USER_NAME}/.config/icarmeter 2>/dev/null || true
chmod 666 /home/${FIRST_USER_NAME}/.config/icarmeter/config.toml 2>/dev/null || true
touch /var/log/icarmeter-bluetooth.log && chmod 666 /var/log/icarmeter-bluetooth.log 2>/dev/null || true

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
systemctl enable icarmeter-api.service || true
