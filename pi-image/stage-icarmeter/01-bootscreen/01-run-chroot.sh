#!/bin/bash -e
# ==============================================================================
# Chroot-side: Activate Plymouth theme and configure silent boot parameters
# ==============================================================================

# Set default Plymouth splash theme and update initramfs
if command -v plymouth-set-default-theme > /dev/null 2>&1; then
    plymouth-set-default-theme -R icarmeter || true
fi

# Locate cmdline.txt (/boot/firmware/cmdline.txt on Bookworm, /boot/cmdline.txt on Bullseye)
CMDLINE_FILE=""
if [ -f "/boot/firmware/cmdline.txt" ]; then
    CMDLINE_FILE="/boot/firmware/cmdline.txt"
elif [ -f "/boot/cmdline.txt" ]; then
    CMDLINE_FILE="/boot/cmdline.txt"
fi

if [ -n "$CMDLINE_FILE" ]; then
    # 1. Remove console=serial0,115200 so kernel serial console doesn't conflict with GPS
    sed -i -E 's/console=serial0,[0-9]+[ ]*//g' "$CMDLINE_FILE"

    # 2. Append silent, cursor-hiding, and blanking-disabled kernel parameters
    for param in "quiet" "splash" "loglevel=1" "systemd.show_status=0" "logo.nologo" "vt.global_cursor_default=0" "consoleblank=0" "plymouth.ignore-serial-consoles"; do
        if ! grep -q "$param" "$CMDLINE_FILE"; then
            sed -i "s/$/ $param/" "$CMDLINE_FILE"
        fi
    done
fi

# 3. Configure Plymouth to retain splash on quit (seamless handover to dashboard, no black flicker)
mkdir -p /etc/systemd/system/plymouth-quit.service.d
cat << 'EOF' > /etc/systemd/system/plymouth-quit.service.d/override.conf
[Service]
ExecStart=
ExecStart=-/usr/bin/plymouth --quit --retain-splash
EOF

mkdir -p /etc/systemd/system/plymouth-quit-wait.service.d
cat << 'EOF' > /etc/systemd/system/plymouth-quit-wait.service.d/override.conf
[Service]
ExecStart=
ExecStart=-/usr/bin/plymouth --quit --retain-splash
EOF

# Locate config.txt (/boot/firmware/config.txt on Bookworm, /boot/config.txt on Bullseye)
CONFIG_FILE=""
if [ -f "/boot/firmware/config.txt" ]; then
    CONFIG_FILE="/boot/firmware/config.txt"
elif [ -f "/boot/config.txt" ]; then
    CONFIG_FILE="/boot/config.txt"
fi

if [ -n "$CONFIG_FILE" ]; then
    # Remove disable-bt overlay so onboard Bluetooth hardware remains enabled
    sed -i '/disable-bt/d' "$CONFIG_FILE" 2>/dev/null || true
    sed -i '/dtparam=bluetooth=on/d' "$CONFIG_FILE" 2>/dev/null || true

    if ! grep -q "ICarMeter" "$CONFIG_FILE"; then
        cat << 'EOF' >> "$CONFIG_FILE"

# --- ICarMeter Hardware Configuration ---
disable_splash=1
boot_delay=0
enable_uart=1
dtoverlay=miniuart-bt
core_freq=250
core_freq_min=250
framebuffer_depth=32
framebuffer_ignore_alpha=1
EOF
    else
        # Ensure miniuart-bt and core frequency locking exist so Bluetooth controller is active alongside GPS serial
        if ! grep -q "dtoverlay=miniuart-bt" "$CONFIG_FILE"; then
            echo "dtoverlay=miniuart-bt" >> "$CONFIG_FILE"
        fi
        if ! grep -q "core_freq=250" "$CONFIG_FILE"; then
            echo "core_freq=250" >> "$CONFIG_FILE"
            echo "core_freq_min=250" >> "$CONFIG_FILE"
        fi
    fi
fi
