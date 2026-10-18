#!/bin/bash
# ==============================================================================
# ICarMeter Always-On Bluetooth Configuration
# Device Name: ICarMeter by I4104
# Broadcasts both Classic Bluetooth Inquiry & BLE Advertisement
# ==============================================================================

echo "[Bluetooth-Init] Initializing Bluetooth as 'ICarMeter by I4104'..."

# 1. Unblock all wireless radios via rfkill
rfkill unblock all 2>/dev/null || true
rfkill unblock bluetooth 2>/dev/null || true

# 2. Trigger Broadcom bcm43xx firmware attachment if on Raspberry Pi
systemctl start hciuart.service 2>/dev/null || true
systemctl start bcm43xx.service 2>/dev/null || true

# 3. Wait for Bluetooth adapter (hci0) to appear (up to 30s)
for i in {1..30}; do
    if hciconfig hci0 2>/dev/null; then
        echo "[Bluetooth-Init] Found hci0 on iteration $i."
        break
    fi
    sleep 1
done

if ! hciconfig hci0 2>/dev/null; then
    echo "[Bluetooth-Init] WARNING: hci0 not found after 30s. Checking dmesg..."
    dmesg | grep -i bluetooth | tail -n 10 || true
fi

# 4. Bring up hci0 interface
hciconfig hci0 up 2>/dev/null || true
sleep 1

# 5. Configure via BlueZ Management interface (btmgmt)
if command -v btmgmt > /dev/null 2>&1; then
    btmgmt -i hci0 power off 2>/dev/null || true
    btmgmt -i hci0 le on 2>/dev/null || true
    btmgmt -i hci0 bredr on 2>/dev/null || true
    btmgmt -i hci0 connectable on 2>/dev/null || true
    btmgmt -i hci0 discov on 2>/dev/null || true
    btmgmt -i hci0 name "ICarMeter by I4104" 2>/dev/null || true
    btmgmt -i hci0 power on 2>/dev/null || true
fi

# 6. Configure via hciconfig and bluetoothctl
hciconfig hci0 name "ICarMeter by I4104" 2>/dev/null || true
bluetoothctl system-alias "ICarMeter by I4104" 2>/dev/null || true
bluetoothctl power on 2>/dev/null || true
bluetoothctl discoverable-timeout 0 2>/dev/null || true
bluetoothctl pairable-timeout 0 2>/dev/null || true
bluetoothctl discoverable on 2>/dev/null || true
bluetoothctl pairable on 2>/dev/null || true

# 7. Enable Classic BT Page & Inquiry Scan (Visible to phone Settings -> Bluetooth)
hciconfig hci0 piscan 2>/dev/null || true

# 8. Set Hardware-Level LE Advertising Data with Complete Local Name:
# Name "ICarMeter by I4104" (18 bytes: 49 43 61 72 4d 65 74 65 72 20 62 79 20 49 34 31 30 34)
# Flags: 02 01 06 (LE General Discoverable)
# Complete Local Name: 13 09 ...
# Total AD payload: 23 bytes (0x17)
hciconfig hci0 noleadv 2>/dev/null || true
hcitool -i hci0 cmd 0x08 0x0008 17 02 01 06 13 09 49 43 61 72 4d 65 74 65 72 20 62 79 20 49 34 31 30 34 00 00 00 00 00 00 00 00 2>/dev/null || true
hcitool -i hci0 cmd 0x08 0x0009 17 02 01 06 13 09 49 43 61 72 4d 65 74 65 72 20 62 79 20 49 34 31 30 34 00 00 00 00 00 00 00 00 2>/dev/null || true
hciconfig hci0 leadv 0 2>/dev/null || true

echo "[Bluetooth-Init] Bluetooth 'ICarMeter by I4104' is active, discoverable, and broadcasting."

# 9. Launch Python BLE GATT Server in background if script exists
if [ -f /opt/icarmeter/icarmeter-ble-service.py ]; then
    python3 /opt/icarmeter/icarmeter-ble-service.py &
    BLE_PID=$!
fi

# 10. Continuous keep-alive loop to prevent sleep or timeout
while true; do
    sleep 15
    hciconfig hci0 name "ICarMeter by I4104" 2>/dev/null || true
    hciconfig hci0 piscan 2>/dev/null || true
    hciconfig hci0 leadv 0 2>/dev/null || true
    bluetoothctl power on 2>/dev/null || true
    bluetoothctl discoverable on 2>/dev/null || true

    # Restart BLE GATT daemon if it died
    if [ -f /opt/icarmeter/icarmeter-ble-service.py ]; then
        if [ -z "$BLE_PID" ] || ! kill -0 "$BLE_PID" 2>/dev/null; then
            python3 /opt/icarmeter/icarmeter-ble-service.py &
            BLE_PID=$!
        fi
    fi
done
