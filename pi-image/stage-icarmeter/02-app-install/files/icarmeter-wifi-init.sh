#!/bin/bash
# ==============================================================================
# ICarMeter Wi-Fi Hotspot — Always-On Init & Watchdog
# Runs as the main process of icarmeter-hotspot.service (Type=simple).
#
# This script guarantees the AP hotspot stays alive so the mobile companion
# app can connect at any time. This script owns activation and recovery so
# NetworkManager callbacks do not issue competing connection-up requests.
# ==============================================================================
set -u

CONN_NAME="ICarMeter-Hotspot"
WIFI_IFACE="wlan0"
HOTSPOT_IP="192.168.4.1"
MARKER="/dev/shm/icarmeter_hotspot_active"
API_PORT=8080

# ─── Helpers ──────────────────────────────────────────────────────────────────
log() { echo "[WiFi-Hotspot] $*"; }

ensure_wifi_unblocked() {
    rfkill unblock wifi  2>/dev/null || true
    rfkill unblock wlan  2>/dev/null || true
    nmcli radio wifi on  2>/dev/null || true
}

ensure_power_save_off() {
    iw dev "$WIFI_IFACE" set power_save off 2>/dev/null || true
}

wait_for_wifi() {
    local attempt
    for attempt in $(seq 1 60); do
        [ -e "/sys/class/net/$WIFI_IFACE" ] && return 0
        sleep 1
    done
    log "WARNING: $WIFI_IFACE did not appear after 60s."
    return 1
}

ensure_iptables_open() {
    # Allow ICMP ping (essential for mobile devices to verify gateway health instantly)
    iptables -C INPUT -i wlan0 -p icmp -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p icmp -j ACCEPT 2>/dev/null || true
    # Allow port 8080 (API)
    iptables -C INPUT -i wlan0 -p tcp --dport "$API_PORT" -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p tcp --dport "$API_PORT" -j ACCEPT 2>/dev/null || true
    # Allow port 80 (Standard HTTP & captive portal)
    iptables -C INPUT -i wlan0 -p tcp --dport 80 -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p tcp --dport 80 -j ACCEPT 2>/dev/null || true
    # Allow DNS (UDP & TCP 53)
    iptables -C INPUT -i wlan0 -p udp --dport 53 -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p udp --dport 53 -j ACCEPT 2>/dev/null || true
    iptables -C INPUT -i wlan0 -p tcp --dport 53 -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p tcp --dport 53 -j ACCEPT 2>/dev/null || true
    # Allow DHCP (UDP 67 server & 68 client)
    iptables -C INPUT -i wlan0 -p udp --dport 67 -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p udp --dport 67 -j ACCEPT 2>/dev/null || true
    iptables -C INPUT -i wlan0 -p udp --dport 68 -j ACCEPT 2>/dev/null \
        || iptables -I INPUT -i wlan0 -p udp --dport 68 -j ACCEPT 2>/dev/null || true
    # Backup NAT redirect: Forward incoming port 80 to 8080 if port 80 is not natively bound
    iptables -t nat -C PREROUTING -i wlan0 -p tcp --dport 80 -j REDIRECT --to-ports "$API_PORT" 2>/dev/null \
        || iptables -t nat -I PREROUTING -i wlan0 -p tcp --dport 80 -j REDIRECT --to-ports "$API_PORT" 2>/dev/null || true
}

activate_hotspot() {
    local attempt delay max_retries=8
    for attempt in $(seq 1 $max_retries); do
        if nmcli -t -f NAME connection show --active 2>/dev/null | grep -Fxq "$CONN_NAME"; then
            return 0
        fi
        log "Activating $CONN_NAME (attempt $attempt/$max_retries)..."
        nmcli --wait 20 connection up "$CONN_NAME" ifname "$WIFI_IFACE" 2>/dev/null && return 0
        delay=$((attempt < 5 ? attempt * 2 : 10))
        sleep "$delay"
    done
    log "WARNING: Failed to activate $CONN_NAME after $max_retries attempts."
    return 1
}

# Verify the hotspot interface actually has the correct IP bound (with retry buffer to avoid restart flapping)
verify_ip_assigned() {
    local attempt
    for attempt in $(seq 1 5); do
        if ip -4 -o addr show dev "$WIFI_IFACE" 2>/dev/null | grep -Fq "$HOTSPOT_IP/24"; then
            return 0
        fi
        sleep 1
    done
    log "WARNING: wlan0 missing $HOTSPOT_IP/24 after 5s — forcing hotspot restart..."
    nmcli --wait 10 connection down "$CONN_NAME" 2>/dev/null || true
    sleep 2
    nmcli --wait 20 connection up "$CONN_NAME" ifname "$WIFI_IFACE" 2>/dev/null || true
}

# ─── Clean shutdown ──────────────────────────────────────────────────────────
trap 'rm -f "$MARKER" 2>/dev/null; log "Service stopping."; exit 0' TERM INT

# ─── Phase 1: Initial Setup ──────────────────────────────────────────────────
log "Starting Wi-Fi Hotspot always-on service..."

ensure_wifi_unblocked
wait_for_wifi || true

# Wait for NetworkManager to become available (up to 30s)
for _ in $(seq 1 30); do
    nmcli general status >/dev/null 2>&1 && break
    sleep 1
done

ensure_power_save_off

# Deactivate any competing wifi connections on wlan0 (station-mode profiles)
for conn in $(nmcli -t -f NAME,TYPE,DEVICE connection show --active 2>/dev/null \
              | grep ':802-11-wireless:wlan0' | cut -d: -f1); do
    if [ "$conn" != "$CONN_NAME" ]; then
        log "Deactivating competing connection: $conn"
        nmcli connection down "$conn" 2>/dev/null || true
    fi
done

activate_hotspot
ensure_iptables_open
verify_ip_assigned

touch "$MARKER" 2>/dev/null || true
log "Hotspot is UP. Entering always-alive watchdog loop..."

# ─── Phase 2: Always-Alive Watchdog Loop ─────────────────────────────────────
# Runs every 8 seconds in foreground. systemd manages this process lifecycle.
# IMPORTANT: Never call power_save or rfkill repeatedly while hotspot is active,
# as Broadcom chip resets drop connected clients.
while true; do
    sleep 5

    # Check if hotspot is still active
    if ! nmcli -t -f NAME connection show --active 2>/dev/null | grep -Fxq "$CONN_NAME" \
       || ! ip -4 -o addr show dev "$WIFI_IFACE" 2>/dev/null | grep -Fq "$HOTSPOT_IP/24"; then
        log "Hotspot DOWN detected — re-activating..."
        ensure_wifi_unblocked
        ensure_power_save_off
        wait_for_wifi || true
        activate_hotspot
        ensure_iptables_open
        verify_ip_assigned
    fi

    # Refresh marker timestamp for dashboard status probe
    touch "$MARKER" 2>/dev/null || true
done
