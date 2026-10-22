#!/bin/bash
# ==============================================================================
# Initialize the onboard Bluetooth controller for TPMS and RFCOMM/SPP.
# ==============================================================================
set -u

BT_LOG_FILE="/var/log/icarmeter-bluetooth.log"
BT_LOG_FALLBACK="/dev/shm/icarmeter-bluetooth.log"

# Clear old logs on each system boot / service initialization
: > "${BT_LOG_FILE}" 2>/dev/null || true
: > "${BT_LOG_FALLBACK}" 2>/dev/null || true
chmod 666 "${BT_LOG_FILE}" "${BT_LOG_FALLBACK}" 2>/dev/null || true

log_bt() {
    local msg="[$(date '+%Y-%m-%d %H:%M:%S')] [Bluetooth-Init] $1"
    echo "${msg}"
    echo "${msg}" >> "${BT_LOG_FILE}" 2>/dev/null || echo "${msg}" >> "${BT_LOG_FALLBACK}" 2>/dev/null || true
}

log_bt "System start: Bluetooth subsystem initializing..."
log_bt "Initializing Bluetooth adapter for TPMS sensor scanning and SPP..."
rfkill unblock bluetooth 2>/dev/null || true

controller_found=0
uart_restart_attempted=0
for i in $(seq 1 60); do
    if /usr/bin/bluetoothctl list 2>/dev/null | grep -q "Controller" || \
       /usr/bin/btmgmt info 2>/dev/null | grep -Eq "hci[0-9]+:"; then
        controller_found=1
        break
    fi
    # Restart hciuart only once if the controller has not appeared yet.
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

# Power on the controller so the raw HCI socket in icarmeter can capture TPMS LE advertisements
rfkill unblock bluetooth 2>/dev/null || true
for attempt in $(seq 1 5); do
    /usr/bin/bluetoothctl power on >/dev/null 2>&1 && break
    sleep 2
done

# Set system Bluetooth alias and controller name to ICarMeter
/usr/bin/bluetoothctl system-alias "ICarMeter" 2>/dev/null || true

# Direct kernel-level Bluetooth controller configuration via btmgmt
if [ -x /usr/bin/btmgmt ]; then
    /usr/bin/btmgmt --index 0 power on 2>/dev/null || /usr/bin/btmgmt power on 2>/dev/null || true
    /usr/bin/btmgmt --index 0 name "ICarMeter" "ICarMeter" 2>/dev/null || /usr/bin/btmgmt name "ICarMeter" "ICarMeter" 2>/dev/null || true
    /usr/bin/btmgmt --index 0 class 0x200404 2>/dev/null || /usr/bin/btmgmt class 0x200404 2>/dev/null || true
    /usr/bin/btmgmt --index 0 connectable on 2>/dev/null || /usr/bin/btmgmt connectable on 2>/dev/null || true
    /usr/bin/btmgmt --index 0 discov on 2>/dev/null || /usr/bin/btmgmt discov on 2>/dev/null || true
    /usr/bin/btmgmt --index 0 pairable on 2>/dev/null || /usr/bin/btmgmt pairable on 2>/dev/null || true
fi

# Controller is permanently discoverable and pairable (RFCOMM/SPP)
# Never times out, channel is always ready for phone connection
/usr/bin/bluetoothctl discoverable-timeout 0 2>/dev/null || true
/usr/bin/bluetoothctl pairable-timeout 0 2>/dev/null || true
/usr/bin/bluetoothctl discoverable on 2>/dev/null || true
/usr/bin/bluetoothctl pairable on 2>/dev/null || true

# Force hardware-level Page Scan & Inquiry Scan so mobile phones can discover the Pi
if [ -x /usr/bin/hciconfig ]; then
    /usr/bin/hciconfig hci0 up 2>/dev/null || true
    /usr/bin/hciconfig hci0 piscan 2>/dev/null || true
fi

# Register Serial Port Profile (SPP) in BlueZ SDP
# Wait for SDP socket to become available from bluetoothd --compat
sdp_socket_ready=0
for i in $(seq 1 20); do
    if [ -S /var/run/sdp ] || [ -S /run/sdp ]; then
        chmod 777 /var/run/sdp /run/sdp 2>/dev/null || true
        sdp_socket_ready=1
        break
    fi
    sleep 0.5
done

if [ "$sdp_socket_ready" -eq 1 ]; then
    if sdptool add --channel=1 SP >/dev/null 2>&1 || sdptool add SP >/dev/null 2>&1; then
        log_bt "Registered Serial Port Profile (SPP) on SDP channel 1."
    else
        log_bt "Warning: Failed to register SPP via sdptool."
    fi
else
    log_bt "Warning: SDP socket (/var/run/sdp) not found after 10s. Compat mode may not be active."
fi

log_bt "Controller powered, named ICarMeter, and permanently listening for RFCOMM SPP connections."
