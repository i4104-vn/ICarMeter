#!/bin/bash -e
# ==============================================================================
# Host-side: Install ICarMeter binary, configuration, and boot integration
# ==============================================================================

APP_DIR="${ROOTFS_DIR}/opt/icarmeter"
install -d -m 755 "${APP_DIR}"
install -m 666 files/config.toml "${APP_DIR}/config.toml"
install -m 755 files/icarmeter-bt-init.sh "${APP_DIR}/icarmeter-bt-init.sh"
install -m 755 files/icarmeter-wifi-init.sh "${APP_DIR}/icarmeter-wifi-init.sh"
install -m 755 files/icarmeter-api.py "${APP_DIR}/icarmeter-api.py"
install -m 755 files/icarmeter-bt-agent.py "${APP_DIR}/icarmeter-bt-agent.py"
install -d -m 755 "${ROOTFS_DIR}/etc/profile.d"
install -m 644 files/icarmeter-dashboard.sh "${ROOTFS_DIR}/etc/profile.d/icarmeter-dashboard.sh"
install -m 644 files/icarmeter-bluetooth.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-bluetooth.service"
install -m 644 files/icarmeter-bt-agent.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-bt-agent.service"
install -m 644 files/icarmeter-hotspot.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-hotspot.service"
install -m 644 files/icarmeter-api.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-api.service"
install -d -m 700 "${ROOTFS_DIR}/etc/NetworkManager/system-connections"
install -m 600 files/icarmeter-hotspot.nmconnection "${ROOTFS_DIR}/etc/NetworkManager/system-connections/icarmeter-hotspot.nmconnection"
install -d -m 755 "${ROOTFS_DIR}/etc/NetworkManager/dispatcher.d"
install -m 755 files/99-icarmeter-hotspot "${ROOTFS_DIR}/etc/NetworkManager/dispatcher.d/99-icarmeter-hotspot"
install -d -m 755 "${ROOTFS_DIR}/etc/NetworkManager/dnsmasq-shared.d"
install -m 644 files/icarmeter-dnsmasq.conf "${ROOTFS_DIR}/etc/NetworkManager/dnsmasq-shared.d/icarmeter.conf"

# Install Mobile Companion Web Application into /opt/icarmeter/www
install -d -m 755 "${APP_DIR}/www"
if [ -d files/www ]; then
    cp -rf files/www/* "${APP_DIR}/www/"
fi

if [ -f files/car_dashboard ]; then
    echo "Found pre-compiled car_dashboard binary. Installing into rootfs..."
    install -m 755 files/car_dashboard "${APP_DIR}/car_dashboard"
else
    echo "ERROR: Pre-compiled binary 'files/car_dashboard' not found!" >&2
    echo "Please build the Rust binary first (e.g. via cargo build --release --target aarch64-unknown-linux-gnu)" >&2
    echo "and place it at 'stage-icarmeter/02-app-install/files/car_dashboard' before building the OS image." >&2
    exit 1
fi
