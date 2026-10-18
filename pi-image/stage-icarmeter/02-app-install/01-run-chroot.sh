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

# Configure Bluetooth: Set name to "ICarMeter" and keep always discoverable/powered on
mkdir -p /etc/bluetooth
cat << 'EOF' > /etc/bluetooth/main.conf
[General]
Name = ICarMeter
Class = 0x000100
DiscoverableTimeout = 0
PairableTimeout = 0
AutoEnable = true
ControllerMode = dual
FastConnectable = true
JustWorksRepairing = always

[Policy]
AutoEnable = true
EOF

# Set system machine-info pretty hostname for Bluetooth discovery
echo 'PRETTY_HOSTNAME="ICarMeter"' > /etc/machine-info

# Ensure bluetooth scripts have execute permission
chmod 755 /opt/icarmeter/icarmeter-bt-init.sh 2>/dev/null || true
chmod 755 /opt/icarmeter/icarmeter-ble-service.py 2>/dev/null || true

# Offline mode optimizations: do not block or wait for WiFi/network on boot
systemctl disable systemd-networkd-wait-online.service 2>/dev/null || true
systemctl disable NetworkManager-wait-online.service 2>/dev/null || true

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
systemctl enable icarmeter-ble.service || true
