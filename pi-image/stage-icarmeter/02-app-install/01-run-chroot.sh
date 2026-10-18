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

# Enable autostart service on boot
systemctl daemon-reload || true
systemctl enable icarmeter.service || true
