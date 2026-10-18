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

# Configure Bluetooth: Set name to "ICarMeter by I4104" and keep always discoverable/powered on
mkdir -p /etc/bluetooth
if [ -f /etc/bluetooth/main.conf ]; then
    sed -i 's/^#*Name = .*/Name = ICarMeter by I4104/' /etc/bluetooth/main.conf
    sed -i 's/^#*DiscoverableTimeout = .*/DiscoverableTimeout = 0/' /etc/bluetooth/main.conf
    sed -i 's/^#*PairableTimeout = .*/PairableTimeout = 0/' /etc/bluetooth/main.conf
    sed -i 's/^#*AutoEnable = .*/AutoEnable = true/' /etc/bluetooth/main.conf
fi
if ! grep -q "ICarMeter by I4104" /etc/bluetooth/main.conf 2>/dev/null; then
    cat << 'EOF' >> /etc/bluetooth/main.conf

[General]
Name = ICarMeter by I4104
DiscoverableTimeout = 0
PairableTimeout = 0
AutoEnable = true

[Policy]
AutoEnable = true
EOF
fi

# Set system machine-info pretty hostname for Bluetooth discovery
echo 'PRETTY_HOSTNAME="ICarMeter by I4104"' > /etc/machine-info

# Ensure bluetooth scripts have execute permission
chmod 755 /opt/icarmeter/icarmeter-bt-init.sh 2>/dev/null || true

# Offline mode optimizations: do not block or wait for WiFi/network on boot
systemctl disable systemd-networkd-wait-online.service 2>/dev/null || true
systemctl disable NetworkManager-wait-online.service 2>/dev/null || true
systemctl mask getty@tty1.service 2>/dev/null || true

# Enable autostart services on boot
systemctl daemon-reload || true
systemctl enable icarmeter.service || true
systemctl enable icarmeter-bluetooth.service || true
