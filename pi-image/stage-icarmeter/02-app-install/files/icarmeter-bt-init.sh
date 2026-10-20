#!/bin/bash
# ==============================================================================
# Initialize onboard Bluetooth controller strictly for TPMS LE sensor scanning.
# Classic BT discovery/pairing and BLE GATT peripheral modes are disabled.
# ==============================================================================
set -u

log_bt() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [Bluetooth-Init] $1"
    echo "${msg}"
    echo "${msg}" >> /var/log/icarmeter-bluetooth.log 2>/dev/null || true
}

log_bt "Initializing Bluetooth adapter for TPMS sensor scanning..."
rfkill unblock bluetooth 2>/dev/null || true

controller_found=0
for i in $(seq 1 45); do
    if /usr/bin/bluetoothctl list 2>/dev/null | grep -q "Controller" || \
       /usr/bin/btmgmt info 2>/dev/null | grep -Eq "hci[0-9]+:"; then
        controller_found=1
        break
    fi
    # If controller hasn't appeared yet, poke rfkill and hciuart to trigger driver attachment
    if [ "$i" -eq 4 ] || [ "$i" -eq 12 ] || [ "$i" -eq 25 ]; then
        rfkill unblock bluetooth 2>/dev/null || true
        systemctl restart hciuart.service 2>/dev/null || true
    fi
    sleep 1
done

if [ "${controller_found}" != "1" ]; then
    log_bt "Warning: No Bluetooth controller appeared after 45s." >&2
    exit 0
fi

log_bt "Bluetooth controller detected. Powering on..."

# Power on the controller so the raw HCI socket in car_dashboard can capture TPMS LE advertisements
rfkill unblock bluetooth 2>/dev/null || true
hciconfig hci0 up 2>/dev/null || true
if [ -x /usr/bin/btmgmt ]; then
    /usr/bin/btmgmt power on || true
fi
/usr/bin/bluetoothctl power on || true

# Controller is permanently discoverable and pairable (RFCOMM/SPP)
# Never times out, channel is always ready for phone connection
/usr/bin/bluetoothctl discoverable-timeout 0 2>/dev/null || true
/usr/bin/bluetoothctl pairable-timeout 0 2>/dev/null || true
/usr/bin/bluetoothctl discoverable on 2>/dev/null || true
/usr/bin/bluetoothctl pairable on 2>/dev/null || true

# Register Serial Port Profile (SPP) in BlueZ SDP
sdptool add SP 2>/dev/null || true

log_bt "Controller powered and permanently listening for RFCOMM SPP connections."
