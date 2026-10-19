#!/bin/bash -e
# ==============================================================================
# Host-side: Install ICarMeter binary, configuration, and boot integration
# ==============================================================================

APP_DIR="${ROOTFS_DIR}/opt/icarmeter"
install -d -m 755 "${APP_DIR}"
install -m 644 files/config.toml "${APP_DIR}/config.toml"
install -m 755 files/icarmeter-bt-init.sh "${APP_DIR}/icarmeter-bt-init.sh"
install -m 755 files/icarmeter-wifi-init.sh "${APP_DIR}/icarmeter-wifi-init.sh"
install -m 755 files/icarmeter-api.py "${APP_DIR}/icarmeter-api.py"
install -d -m 755 "${ROOTFS_DIR}/etc/profile.d"
install -m 644 files/icarmeter-dashboard.sh "${ROOTFS_DIR}/etc/profile.d/icarmeter-dashboard.sh"
install -m 644 files/icarmeter-bluetooth.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-bluetooth.service"
install -m 644 files/icarmeter-hotspot.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-hotspot.service"
install -m 644 files/icarmeter-api.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-api.service"
install -d -m 700 "${ROOTFS_DIR}/etc/NetworkManager/system-connections"
install -m 600 files/icarmeter-hotspot.nmconnection "${ROOTFS_DIR}/etc/NetworkManager/system-connections/icarmeter-hotspot.nmconnection"

if [ -f files/car_dashboard ]; then
    echo "Found pre-compiled car_dashboard binary. Installing into rootfs..."
    install -m 755 files/car_dashboard "${APP_DIR}/car_dashboard"
else
    echo "ERROR: Pre-compiled binary 'files/car_dashboard' not found!" >&2
    echo "Please build the Rust binary first (e.g. via cargo build --release --target aarch64-unknown-linux-gnu)" >&2
    echo "and place it at 'stage-icarmeter/02-app-install/files/car_dashboard' before building the OS image." >&2
    exit 1
fi
