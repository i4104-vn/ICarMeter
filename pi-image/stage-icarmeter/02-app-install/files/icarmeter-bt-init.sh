#!/bin/bash
# ==============================================================================
# ICarMeter Always-On Bluetooth Configuration
# Device Name: ICarMeter by I4104
# Ensures Bluetooth is powered on, discoverable, and advertising BLE continuously
# ==============================================================================

echo "[Bluetooth-Init] Initializing ICarMeter Bluetooth as 'ICarMeter by I4104'..."

# Unblock radio
rfkill unblock bluetooth 2>/dev/null || true

# Wait for Bluetooth adapter (hci0) to appear (up to 30s)
for i in {1..30}; do
    if hciconfig hci0 2>/dev/null; then
        break
    fi
    sleep 1
done

# Bring up hci0 interface
hciconfig hci0 up 2>/dev/null || true
sleep 1

# Configure device name & aliases
hciconfig hci0 name "ICarMeter by I4104" 2>/dev/null || true
bluetoothctl system-alias "ICarMeter by I4104" 2>/dev/null || true

# Enable Classic BT Page & Inquiry Scan (Makes device discoverable)
hciconfig hci0 piscan 2>/dev/null || true

# Enable Bluetoothctl always-discoverable & always-pairable (no timeout)
bluetoothctl power on 2>/dev/null || true
bluetoothctl discoverable-timeout 0 2>/dev/null || true
bluetoothctl pairable-timeout 0 2>/dev/null || true
bluetoothctl discoverable on 2>/dev/null || true
bluetoothctl pairable on 2>/dev/null || true

# Enable BLE advertising (leadv 0: connectable undirected advertising)
hciconfig hci0 leadv 0 2>/dev/null || true
bluetoothctl advertise on 2>/dev/null || true

echo "[Bluetooth-Init] Bluetooth 'ICarMeter by I4104' is active, powered on, and discoverable."

# Continuous keep-alive loop to prevent sleep or timeout
while true; do
    sleep 15
    hciconfig hci0 name "ICarMeter by I4104" 2>/dev/null || true
    hciconfig hci0 piscan 2>/dev/null || true
    hciconfig hci0 leadv 0 2>/dev/null || true
    bluetoothctl power on 2>/dev/null || true
    bluetoothctl discoverable on 2>/dev/null || true
done
