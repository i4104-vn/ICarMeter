#!/bin/bash
# ==============================================================================
# Unblock Wi-Fi rfkill and ensure NetworkManager Hotspot connection is active
# ==============================================================================
set -u

echo "[WiFi-Init] Unblocking Wi-Fi..."
rfkill unblock wifi 2>/dev/null || true
rfkill unblock wlan 2>/dev/null || true

# Wait for NetworkManager to become available (up to 30s)
for _ in $(seq 1 30); do
    if nmcli general status >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

# Ensure radio wifi is turned on in NetworkManager
nmcli radio wifi on 2>/dev/null || true

# Disable power-saving on Wi-Fi adapter so it never goes to sleep
iw dev wlan0 set power_save off 2>/dev/null || true
iwconfig wlan0 power off 2>/dev/null || true

# Bring up the hotspot profile if it is not already active
if nmcli connection show --active 2>/dev/null | grep -q "ICarMeter-Hotspot"; then
    echo "[WiFi-Init] ICarMeter-Hotspot is already active."
else
    echo "[WiFi-Init] Activating ICarMeter-Hotspot connection..."
    nmcli connection up "ICarMeter-Hotspot" 2>/dev/null || true
fi

# Touch marker for Dashboard footer status probe
touch /dev/shm/icarmeter_hotspot_active 2>/dev/null || true

# Run background persistent watchdog loop to keep Hotspot ALWAYS ON
(
    while true; do
        sleep 10
        rfkill unblock wifi 2>/dev/null || true
        nmcli radio wifi on 2>/dev/null || true
        iw dev wlan0 set power_save off 2>/dev/null || true
        if ! nmcli connection show --active 2>/dev/null | grep -q "ICarMeter-Hotspot"; then
            echo "[WiFi-Watchdog] Re-activating Hotspot..."
            nmcli connection up "ICarMeter-Hotspot" 2>/dev/null || true
        fi
        touch /dev/shm/icarmeter_hotspot_active 2>/dev/null || true
    done
) &
echo "[WiFi-Init] Hotspot keeper started successfully."
