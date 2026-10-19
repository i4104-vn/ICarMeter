#!/bin/bash
# ==============================================================================
# Initialize onboard Bluetooth controller strictly for TPMS LE sensor scanning.
# Classic BT discovery/pairing and BLE GATT peripheral modes are disabled.
# ==============================================================================
set -u

echo "[Bluetooth-Init] Initializing Bluetooth adapter for TPMS sensor scanning..."
rfkill unblock bluetooth 2>/dev/null || true

controller_found=0
for _ in $(seq 1 45); do
    if /usr/bin/bluetoothctl list 2>/dev/null | grep -q "Controller" || \
       /usr/bin/btmgmt info 2>/dev/null | grep -Eq "hci[0-9]+:"; then
        controller_found=1
        break
    fi
    sleep 1
done

if [ "${controller_found}" != "1" ]; then
    echo "[Bluetooth-Init] Warning: No Bluetooth controller appeared after 45s." >&2
    exit 0
fi

# Power on the controller so the raw HCI socket in car_dashboard can capture TPMS LE advertisements
if [ -x /usr/bin/btmgmt ]; then
    /usr/bin/btmgmt power on || true
fi
/usr/bin/bluetoothctl power on || true

# Turn off discoverable and pairable: Pi does NOT act as a Bluetooth peripheral for phones.
# Bluetooth is solely dedicated to receiving TPMS sensor broadcasts.
/usr/bin/bluetoothctl discoverable off 2>/dev/null || true
/usr/bin/bluetoothctl pairable off 2>/dev/null || true

echo "[Bluetooth-Init] Controller powered and configured for TPMS LE scanning."
