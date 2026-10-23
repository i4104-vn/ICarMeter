#!/bin/bash -e
# Install the dashboard binary, runtime configuration, and TTY launcher.

APP_DIR="${ROOTFS_DIR}/opt/icarmeter"
install -d -m 755 "${APP_DIR}"
install -m 644 files/config.toml "${APP_DIR}/config.toml"
install -d -m 755 "${ROOTFS_DIR}/etc/profile.d"
install -m 644 files/icarmeter-dashboard.sh "${ROOTFS_DIR}/etc/profile.d/icarmeter-dashboard.sh"

if [ -f files/icarmeter ]; then
    echo "Installing precompiled icarmeter binary..."
    install -m 755 files/icarmeter "${APP_DIR}/icarmeter"
else
    echo "ERROR: Precompiled binary 'files/icarmeter' not found!" >&2
    echo "Build the Rust binary for aarch64-unknown-linux-gnu and place it at files/icarmeter." >&2
    exit 1
fi
