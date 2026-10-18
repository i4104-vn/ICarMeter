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

    # 2. Append quiet splash parameters if not already present
    if ! grep -q "quiet splash" "$CMDLINE_FILE"; then
        sed -i 's/$/ quiet splash loglevel=3 logo.nologo vt.global_cursor_default=0 plymouth.ignore-serial-consoles/' "$CMDLINE_FILE"
    fi
fi

# Locate config.txt (/boot/firmware/config.txt on Bookworm, /boot/config.txt on Bullseye)
CONFIG_FILE=""
if [ -f "/boot/firmware/config.txt" ]; then
    CONFIG_FILE="/boot/firmware/config.txt"
elif [ -f "/boot/config.txt" ]; then
    CONFIG_FILE="/boot/config.txt"
fi

if [ -n "$CONFIG_FILE" ]; then
    if ! grep -q "ICarMeter" "$CONFIG_FILE"; then
        cat << 'EOF' >> "$CONFIG_FILE"

# --- ICarMeter Hardware Configuration ---
disable_splash=1
boot_delay=0
enable_uart=1
dtoverlay=miniuart-bt
EOF
    fi
fi
