#!/bin/bash -e
# ==============================================================================
# Chroot-side: Verify binary, configure caps & enable autostart
# ==============================================================================

if [ ! -f /opt/icarmeter/icarmeter ]; then
    echo "ERROR: /opt/icarmeter/icarmeter was not installed into rootfs!" >&2
    exit 1
fi

# Set binary permissions
chmod 755 /opt/icarmeter/icarmeter

# Setup convenience symlink in PATH
ln -sf /opt/icarmeter/icarmeter /usr/local/bin/icarmeter

# Verify dynamic library linkage and execution
echo "=== Verifying dynamic libraries for /opt/icarmeter/icarmeter ==="
ldd /opt/icarmeter/icarmeter || true
if ldd /opt/icarmeter/icarmeter 2>&1 | grep -i "not found"; then
    echo "ERROR: /opt/icarmeter/icarmeter has missing shared libraries in rootfs!" >&2
    exit 1
fi


# Ensure SSH host keys are generated and present so ssh.service never fails
ssh-keygen -A || true

# Set system machine-info pretty hostname
echo 'PRETTY_HOSTNAME="ICarMeter"' > /etc/machine-info

# Ensure application configuration has proper permissions
chmod 666 /opt/icarmeter/config.toml 2>/dev/null || true
mkdir -p /home/${FIRST_USER_NAME}/.config/icarmeter 2>/dev/null || true
chmod 777 /home/${FIRST_USER_NAME}/.config/icarmeter 2>/dev/null || true
chmod 666 /home/${FIRST_USER_NAME}/.config/icarmeter/config.toml 2>/dev/null || true

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

# Disable Bluetooth services to free UART resources exclusively for hardware GPS
systemctl disable bluetooth.service 2>/dev/null || true
systemctl disable hciuart.service 2>/dev/null || true

