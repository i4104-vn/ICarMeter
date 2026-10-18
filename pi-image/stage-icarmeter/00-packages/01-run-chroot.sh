#!/bin/bash -e
# ==============================================================================
# Hardware group permissions & Console autologin configuration
# ==============================================================================

# Add pi user to hardware access groups
for grp in video dialout bluetooth input tty audio render; do
    if getent group "$grp" > /dev/null 2>&1; then
        usermod -a -G "$grp" pi
    fi
done

# 1. Enable console auto-login via raspi-config if available
if command -v raspi-config > /dev/null 2>&1; then
    raspi-config nonint do_boot_behaviour B2 || true
fi

# 2. Explicitly configure systemd getty autologin override for TTY1
mkdir -p /etc/systemd/system/getty@tty1.service.d
cat << 'EOF' > /etc/systemd/system/getty@tty1.service.d/autologin.conf
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin pi --noclear %I $TERM
EOF

systemctl daemon-reload || true
systemctl enable getty@tty1.service || true
