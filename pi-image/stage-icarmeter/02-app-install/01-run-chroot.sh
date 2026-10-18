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

# Offline mode optimizations: do not block or wait for WiFi/network on boot
systemctl disable systemd-networkd-wait-online.service 2>/dev/null || true
systemctl disable NetworkManager-wait-online.service 2>/dev/null || true
systemctl mask getty@tty1.service 2>/dev/null || true

# Enable autostart service on boot
systemctl daemon-reload || true
systemctl enable icarmeter.service || true
