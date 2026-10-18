#!/bin/bash -e
# ==============================================================================
# Host-side: Install ICarMeter binary, configuration, and service unit
# ==============================================================================

APP_DIR="${ROOTFS_DIR}/opt/icarmeter"
install -d -m 755 "${APP_DIR}"
install -m 644 files/config.toml "${APP_DIR}/config.toml"
install -m 755 files/icarmeter-bt-init.sh "${APP_DIR}/icarmeter-bt-init.sh"
install -m 755 files/icarmeter-ble-service.py "${APP_DIR}/icarmeter-ble-service.py"
install -m 644 files/icarmeter.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter.service"
install -m 644 files/icarmeter-bluetooth.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-bluetooth.service"

if [ -f files/car_dashboard ]; then
    echo "Found pre-compiled car_dashboard binary. Installing into rootfs..."
    install -m 755 files/car_dashboard "${APP_DIR}/car_dashboard"
else
    echo "ERROR: Pre-compiled binary 'files/car_dashboard' not found!" >&2
    echo "Please build the Rust binary first (e.g. via cargo build --release --target aarch64-unknown-linux-gnu)" >&2
    echo "and place it at 'stage-icarmeter/02-app-install/files/car_dashboard' before building the OS image." >&2
    exit 1
fi
