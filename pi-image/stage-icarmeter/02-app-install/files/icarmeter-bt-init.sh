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

# Give the shared onboard radio time to establish the Wi-Fi AP first. If Wi-Fi
# is unavailable, continue anyway so Bluetooth TPMS/SPP can still work.
for i in $(seq 1 60); do
    [ -e /dev/shm/icarmeter_hotspot_active ] && break
    sleep 1
done

controller_found=0
uart_restart_attempted=0
for i in $(seq 1 60); do
    if /usr/bin/bluetoothctl list 2>/dev/null | grep -q "Controller" || \
       /usr/bin/btmgmt info 2>/dev/null | grep -Eq "hci[0-9]+:"; then
        controller_found=1
        break
    fi
    # Restart hciuart only once. Repeated resets can keep the shared radio
    # unavailable while Wi-Fi is also bringing up its access point.
    if [ "$i" -eq 15 ] && [ "$uart_restart_attempted" -eq 0 ]; then
        rfkill unblock bluetooth 2>/dev/null || true
        systemctl restart hciuart.service 2>/dev/null || true
        uart_restart_attempted=1
    fi
    sleep 1
done

if [ "${controller_found}" != "1" ]; then
    log_bt "Warning: No Bluetooth controller appeared after 60s." >&2
    exit 0
fi

log_bt "Bluetooth controller detected. Powering on..."

# Power on the controller so the raw HCI socket in car_dashboard can capture TPMS LE advertisements
rfkill unblock bluetooth 2>/dev/null || true
for attempt in $(seq 1 5); do
    /usr/bin/bluetoothctl power on >/dev/null 2>&1 && break
    sleep 2
done

# Set system Bluetooth alias and controller name to ICarMeter
/usr/bin/bluetoothctl system-alias "ICarMeter" 2>/dev/null || true
if [ -x /usr/bin/btmgmt ]; then
    /usr/bin/btmgmt name "ICarMeter" "ICarMeter" 2>/dev/null || true
fi

# Controller is permanently discoverable and pairable (RFCOMM/SPP)
# Never times out, channel is always ready for phone connection
/usr/bin/bluetoothctl discoverable-timeout 0 2>/dev/null || true
/usr/bin/bluetoothctl pairable-timeout 0 2>/dev/null || true
/usr/bin/bluetoothctl discoverable on 2>/dev/null || true
/usr/bin/bluetoothctl pairable on 2>/dev/null || true

# Register Serial Port Profile (SPP) in BlueZ SDP
chmod 777 /var/run/sdp 2>/dev/null || true
sdptool add SP 2>/dev/null || true

log_bt "Controller powered, named ICarMeter, and permanently listening for RFCOMM SPP connections."
