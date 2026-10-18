#!/bin/bash
# Prepare the onboard Bluetooth adapter for Classic BT discovery and BLE GATT.
# The long-running GATT process is managed by icarmeter-ble.service.

set -u

NAME="ICarMeter by I4104"
BLUETOOTHCTL="/usr/bin/bluetoothctl"

echo "[Bluetooth-Init] Waiting for a BlueZ controller..."
rfkill unblock bluetooth 2>/dev/null || true

controller_found=0
for _ in $(seq 1 45); do
    if "${BLUETOOTHCTL}" list 2>/dev/null | grep -q "Controller"; then
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

# Use BlueZ's management API through bluetoothctl. This works for both the
# legacy hciuart adapter and newer Raspberry Pi Bluetooth firmware.
"${BLUETOOTHCTL}" power on || true
"${BLUETOOTHCTL}" system-alias "${NAME}" || true
"${BLUETOOTHCTL}" discoverable-timeout 0 || true
"${BLUETOOTHCTL}" pairable-timeout 0 || true
"${BLUETOOTHCTL}" discoverable on || true
"${BLUETOOTHCTL}" pairable on || true

echo "[Bluetooth-Init] Controller is powered, pairable, and discoverable as '${NAME}'."
