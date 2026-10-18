#!/bin/bash
# Prepare the onboard Bluetooth adapter for Classic BT discovery and BLE GATT.
# The long-running GATT process is managed by icarmeter-ble.service.

set -u

NAME="ICarMeter"
BLUETOOTHCTL="/usr/bin/bluetoothctl"
BTMGMT="/usr/bin/btmgmt"

echo "[Bluetooth-Init] Waiting for a BlueZ controller..."
rfkill unblock bluetooth 2>/dev/null || true

controller_found=0
for _ in $(seq 1 45); do
    if "${BTMGMT}" info 2>/dev/null | grep -Eq "hci[0-9]+:" || \
        "${BLUETOOTHCTL}" list 2>/dev/null | grep -q "Controller"; then
        controller_found=1
        break
    fi
    sleep 1
done

if [ "${controller_found}" != "1" ]; then
    echo "[Bluetooth-Init] No Bluetooth controller appeared after 45s." >&2
    rfkill list bluetooth 2>/dev/null || true
    exit 1
fi

# Bring the controller up through the kernel management API first. This avoids
# relying on the deprecated hciconfig/hcitool utilities and works with both
# Raspberry Pi UART Bluetooth and newer controllers.
if [ -x "${BTMGMT}" ]; then
    "${BTMGMT}" power on || true
    "${BTMGMT}" connectable on || true
    "${BTMGMT}" bondable on || true
    "${BTMGMT}" advertising on || true
fi

# Keep the full human-readable alias for Classic Bluetooth discovery and
# pairing. The BLE GATT advertisement uses a shorter payload-safe name.
"${BLUETOOTHCTL}" power on || true
"${BLUETOOTHCTL}" system-alias "${NAME}" || true
"${BLUETOOTHCTL}" discoverable-timeout 0 || true
"${BLUETOOTHCTL}" pairable-timeout 0 || true
"${BLUETOOTHCTL}" discoverable on || true
"${BLUETOOTHCTL}" pairable on || true

echo "[Bluetooth-Init] Controller is powered, pairable, and discoverable as '${NAME}'."
