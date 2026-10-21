#!/usr/bin/env python3
"""
ICarMeter Headless Wi-Fi Configuration API Service
Listens for JSON API requests from the ICarMeter Mobile App over Wi-Fi Hotspot (192.168.4.1:8080)
to read, write, and synchronize /opt/icarmeter/config.toml in real time.
"""

import base64
import collections
import datetime
import hashlib
import http.server
import json
import os
import re
import socket
import struct
import subprocess
import sys
import threading
import time
import urllib.parse

WS_MAGIC_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"

# Thread-safe WebSocket client registry
connected_ws_clients = set()
ws_clients_lock = threading.Lock()

# Thread-safe Bluetooth RFCOMM client registry
connected_bt_clients = set()
bt_clients_lock = threading.Lock()
RFCOMM_PORT = 1

# ==============================================================================
# Dedicated Bluetooth Operations & bluetoothctl Event Logging Subsystem
# ==============================================================================
BT_LOG_FILE = "/var/log/icarmeter-bluetooth.log"
BT_LOG_FALLBACK = "/dev/shm/icarmeter-bluetooth.log"
bt_logs_lock = threading.Lock()
bt_logs_buffer = collections.deque(maxlen=1000)

def _get_bt_log_path():
    if os.path.exists(BT_LOG_FILE) and os.access(BT_LOG_FILE, os.W_OK):
        return BT_LOG_FILE
    try:
        with open(BT_LOG_FILE, "a") as f:
            pass
        return BT_LOG_FILE
    except Exception:
        return BT_LOG_FALLBACK

def log_bt(source, message):
    """
    Appends a timestamped log entry to the in-memory ring buffer, writes to log file,
    and broadcasts live entry to connected WS and RFCOMM clients.
    """
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    line = f"[{timestamp}] [{source}] {message}"
    with bt_logs_lock:
        bt_logs_buffer.append(line)

    try:
        log_path = _get_bt_log_path()
        with open(log_path, "a", encoding="utf-8") as f:
            f.write(line + "\n")
        # Keep log file bounded under 2MB
        if os.path.getsize(log_path) > 2 * 1024 * 1024:
            try:
                with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
                    file_lines = f.readlines()
                with open(log_path, "w", encoding="utf-8") as f:
                    f.writelines(file_lines[-500:])
            except Exception:
                pass
    except Exception:
        pass

    # Broadcast live entry to both WS and RFCOMM clients
    try:
        ws_broadcast({"type": "bt_log_entry", "line": line})
    except Exception:
        pass
    try:
        bt_broadcast({"type": "bt_log_entry", "line": line})
    except Exception:
        pass

def clear_bt_logs():
    """Clears in-memory buffer and truncates log file."""
    with bt_logs_lock:
        bt_logs_buffer.clear()
    try:
        log_path = _get_bt_log_path()
        with open(log_path, "w", encoding="utf-8") as f:
            f.write("")
    except Exception:
        pass

def init_bt_logs_buffer():
    try:
        log_path = _get_bt_log_path()
        if os.path.exists(log_path):
            with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
                file_lines = f.readlines()
            with bt_logs_lock:
                for l in file_lines[-500:]:
                    clean = l.strip()
                    if clean:
                        bt_logs_buffer.append(clean)
    except Exception:
        pass

def run_bluetoothctl(args, timeout=5):
    """Executes a bluetoothctl command, logging command and output to the dedicated Bluetooth log."""
    cmd_str = "bluetoothctl " + " ".join(args)
    log_bt("bt-exec", f"> {cmd_str}")
    try:
        res = subprocess.run(
            ["bluetoothctl"] + list(args),
            capture_output=True,
            text=True,
            timeout=timeout
        )
        out = (res.stdout or "").strip()
        err = (res.stderr or "").strip()
        if out:
            for line in out.splitlines():
                log_bt("bt-exec", f"  {line.strip()}")
        if err:
            for line in err.splitlines():
                log_bt("bt-exec", f"  [ERR] {line.strip()}")
        return res
    except subprocess.TimeoutExpired:
        log_bt("bt-exec", f"  [TIMEOUT] {cmd_str} timed out after {timeout}s")
        raise
    except Exception as e:
        log_bt("bt-exec", f"  [EXC] {cmd_str} error: {e}")
        raise

def bluetoothctl_monitor_worker():
    """
    Dedicated worker streaming real-time events directly from bluetoothctl.
    Logs BlueZ adapter, device discovery, connection, pairing, and passkey events.
    """
    time.sleep(2)
    log_bt("SYSTEM", "Starting bluetoothctl live event monitor...")
    while True:
        try:
            p = subprocess.Popen(
                ["bluetoothctl"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1
            )
            for line in iter(p.stdout.readline, ""):
                clean = line.strip()
                if not clean:
                    continue
                # Skip interactive prompt lines like '[bluetooth]# '
                clean = re.sub(r'\[bluetooth\s*\]#\s*', '', clean).strip()
                if not clean:
                    continue
                # Strip ANSI color escape sequences
                clean_line = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', clean).strip()
                if clean_line:
                    log_bt("bluetoothctl", clean_line)
            p.wait()
        except Exception as e:
            log_bt("bluetoothctl", f"Monitor restart note: {e}")
        time.sleep(3)

def encode_ws_frame(payload_bytes, opcode=0x1):
    """Encodes server-to-client WebSocket frame (unmasked per RFC 6455)."""
    length = len(payload_bytes)
    b0 = 0x80 | (opcode & 0x0F)
    if length < 126:
        header = bytes([b0, length])
    elif length <= 65535:
        header = bytes([b0, 126]) + struct.pack("!H", length)
    else:
        header = bytes([b0, 127]) + struct.pack("!Q", length)
    return header + payload_bytes

def decode_ws_frame(sock):
    """
    Reads a masked frame from client socket.
    Returns (opcode, payload_bytes) or (None, None) if socket closed/error.
    """
    b = sock.recv(2)
    if not b or len(b) < 2:
        return None, None
    b0, b1 = b[0], b[1]
    opcode = b0 & 0x0F
    is_masked = (b1 & 0x80) != 0
    payload_len = b1 & 0x7F

    if payload_len == 126:
        ext = sock.recv(2)
        if len(ext) < 2:
            return None, None
        payload_len = struct.unpack("!H", ext)[0]
    elif payload_len == 127:
        ext = sock.recv(8)
        if len(ext) < 8:
            return None, None
        payload_len = struct.unpack("!Q", ext)[0]

    mask = sock.recv(4) if is_masked else None
    if is_masked and (not mask or len(mask) < 4):
        return None, None

    chunks = []
    received = 0
    while received < payload_len:
        chunk = sock.recv(min(payload_len - received, 65536))
        if not chunk:
            return None, None
        chunks.append(chunk)
        received += len(chunk)
    raw = b"".join(chunks)

    if is_masked and mask:
        unmasked = bytearray(raw)
        for i in range(len(unmasked)):
            unmasked[i] ^= mask[i % 4]
        return opcode, bytes(unmasked)
    return opcode, raw

def ws_broadcast(msg_dict):
    """Broadcasts a JSON message to all connected WebSocket clients."""
    try:
        data = json.dumps(msg_dict).encode("utf-8")
        frame = encode_ws_frame(data, opcode=0x1)
        with ws_clients_lock:
            dead = []
            for sock in connected_ws_clients:
                try:
                    sock.sendall(frame)
                except Exception:
                    dead.append(sock)
            for d in dead:
                connected_ws_clients.discard(d)
    except Exception as e:
        print(f"[WS] Broadcast error: {e}", file=sys.stderr)

def bt_broadcast(msg_dict):
    """Broadcasts a JSON line message to all connected Bluetooth RFCOMM clients."""
    try:
        data = (json.dumps(msg_dict) + "\n").encode("utf-8")
        with bt_clients_lock:
            dead = []
            for sock in connected_bt_clients:
                try:
                    sock.sendall(data)
                except Exception:
                    dead.append(sock)
            for d in dead:
                connected_bt_clients.discard(d)
    except Exception as e:
        print(f"[BT] Broadcast error: {e}", file=sys.stderr)

def ensure_bt_always_ready():
    """Keeps Bluetooth adapter powered, named ICarMeter, discoverable, and pairable at all times."""
    try:
        subprocess.run(["bluetoothctl", "power", "on"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "system-alias", "ICarMeter"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "discoverable-timeout", "0"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "pairable-timeout", "0"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "discoverable", "on"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "pairable", "on"], capture_output=True, timeout=2)
    except Exception:
        pass

def bt_heartbeat_worker():
    """Background worker: sends periodic heartbeat & TPMS telemetry to connected Bluetooth clients."""
    tick = 0
    while True:
        time.sleep(2)
        tick += 1
        with bt_clients_lock:
            num_bt = len(connected_bt_clients)
        if num_bt > 0:
            touch_app_connected("Bluetooth")
            tpms_data = read_tpms_telemetry()
            bt_broadcast({
                "type": "tpms",
                "data": tpms_data
            })
            if tick % 2 == 0:
                bt_broadcast({
                    "type": "heartbeat",
                    "status": "online",
                    "clients": num_bt,
                    "timestamp": int(time.time())
                })

        # Periodically ensure adapter never goes into sleep/non-discoverable mode
        if tick % 10 == 0:
            ensure_bt_always_ready()

PORT = 8080
def resolve_config_path():
    sudo_user = os.environ.get("SUDO_USER")
    if sudo_user:
        return f"/home/{sudo_user}/.config/icarmeter/config.toml"
    xdg = os.environ.get("XDG_CONFIG_HOME")
    if xdg:
        return os.path.join(xdg, "icarmeter", "config.toml")
    if os.path.isdir("/home/pi"):
        return "/home/pi/.config/icarmeter/config.toml"
    home = os.environ.get("HOME")
    if home and home != "/root":
        return os.path.join(home, ".config", "icarmeter", "config.toml")
    return os.path.expanduser("~/.config/icarmeter/config.toml")

CONFIG_PATH = resolve_config_path()
APP_CONN_MARKER = "/dev/shm/icarmeter_app_connected"
WS_EVENT_MARKER = "/dev/shm/icarmeter_ws_event"
PIN_MARKER = "/dev/shm/icarmeter_bt_pin"
TPMS_TELEMETRY_MARKER = "/dev/shm/icarmeter_tpms_telemetry"

def read_tpms_telemetry():
    if os.path.exists(TPMS_TELEMETRY_MARKER):
        try:
            with open(TPMS_TELEMETRY_MARKER, "r", encoding="utf-8") as f:
                return json.loads(f.read().strip())
        except Exception:
            pass
    return {
        "fl": 0.0, "fr": 0.0, "rl": 0.0, "rr": 0.0,
        "fl_temp": 0.0, "fr_temp": 0.0, "rl_temp": 0.0, "rr_temp": 0.0,
        "alert": 2.2, "high_temp_alert": 70.0
    }

_rfcomm_listening = False
_bt_discovery_active = False
_bt_discovery_lock = threading.Lock()
_bt_discovered_devices = {}
_scan_process = None
_last_pin_timestamp = 0.0

def pin_monitor_worker():
    """Watches /dev/shm/icarmeter_bt_pin and broadcasts bt_pin_code to WiFi clients."""
    global _last_pin_timestamp
    while True:
        try:
            if os.path.exists(PIN_MARKER):
                try:
                    with open(PIN_MARKER, "r", encoding="utf-8") as f:
                        content = f.read().strip()
                    if content:
                        data = json.loads(content)
                        ts = float(data.get("timestamp", 0.0))
                        now = time.time()
                        if ts > _last_pin_timestamp and (now - ts) < 60.0:
                            _last_pin_timestamp = ts
                            pin = str(data.get("pin", "")).strip()
                            mac = str(data.get("mac", "")).strip()
                            if pin:
                                log_bt("BTAgent-PIN", f"Passkey/PIN code required: {pin} for device {mac}")
                                ws_broadcast({
                                    "type": "bt_pin_code",
                                    "pin": pin,
                                    "mac": mac
                                })
                except Exception:
                    pass
        except Exception:
            pass
        time.sleep(0.5)

def is_generic_mac_name(name, mac):
    """Checks if name is empty, identical to MAC, or a MAC-like address (e.g. 74-D2-1D-A3-BB-CC)."""
    if not name or not str(name).strip():
        return True
    clean_n = str(name).strip().replace(":", "").replace("-", "").upper()
    clean_m = str(mac).strip().replace(":", "").replace("-", "").upper()
    if clean_n == clean_m:
        return True
    if re.match(r"^[0-9A-Fa-f]{2}([:-][0-9A-Fa-f]{2}){5}$", str(name).strip()):
        return True
    return False

def get_dbus_device_info(mac):
    """Directly queries org.bluez.Device1 over SystemBus for resolved Name, Alias, and RSSI."""
    try:
        import dbus
        bus = dbus.SystemBus()
        manager = dbus.Interface(bus.get_object("org.bluez", "/"), "org.freedesktop.DBus.ObjectManager")
        objects = manager.GetManagedObjects()
        suffix = "dev_" + mac.replace(":", "_").upper()
        for path, interfaces in objects.items():
            if path.upper().endswith(suffix) and "org.bluez.Device1" in interfaces:
                props = interfaces["org.bluez.Device1"]
                name = str(props.get("Name", "")).strip()
                alias = str(props.get("Alias", "")).strip()
                rssi = int(props.get("RSSI", 0))
                return name, alias, rssi
    except Exception:
        pass
    return None, None, 0

def get_bluetoothctl_device_info(mac):
    """Queries bluetoothctl info for Name, Alias, and RSSI."""
    try:
        res = subprocess.run(["bluetoothctl", "info", mac], capture_output=True, text=True, timeout=2)
        if res.returncode == 0 and res.stdout:
            name = ""
            alias = ""
            rssi = 0
            for line in res.stdout.splitlines():
                line = line.strip()
                m_name = re.match(r"^Name:\s*(.+)$", line)
                if m_name:
                    name = m_name.group(1).strip()
                m_alias = re.match(r"^Alias:\s*(.+)$", line)
                if m_alias:
                    alias = m_alias.group(1).strip()
                m_rssi = re.match(r"^RSSI:\s*([-\d]+)", line)
                if m_rssi:
                    try:
                        rssi = int(m_rssi.group(1))
                    except ValueError:
                        pass
            return name, alias, rssi
    except Exception:
        pass
    return None, None, 0

def resolve_device_name(mac, initial_name=""):
    """
    Attempts to resolve the real user-facing device name (Name or Alias)
    instead of leaving it as an unhelpful MAC address string.
    """
    # 1. Check if initial_name is already a good real name
    if initial_name and not is_generic_mac_name(initial_name, mac):
        return initial_name.strip()

    # 2. Try D-Bus query
    db_name, db_alias, _ = get_dbus_device_info(mac)
    if db_name and not is_generic_mac_name(db_name, mac):
        return db_name
    if db_alias and not is_generic_mac_name(db_alias, mac):
        return db_alias

    # 3. Try bluetoothctl info
    ctl_name, ctl_alias, _ = get_bluetoothctl_device_info(mac)
    if ctl_name and not is_generic_mac_name(ctl_name, mac):
        return ctl_name
    if ctl_alias and not is_generic_mac_name(ctl_alias, mac):
        return ctl_alias

    # 4. Fallback: if we only had initial_name
    if initial_name and str(initial_name).strip():
        return str(initial_name).strip()
    return f"Thiết bị Bluetooth ({mac[-5:]})"

def get_pi_bluetooth_mac():
    """Returns the Bluetooth controller MAC address of the Raspberry Pi."""
    # 1. Standard Linux sysfs
    for hci in ("hci0", "hci1"):
        path = f"/sys/class/bluetooth/{hci}/address"
        if os.path.exists(path):
            try:
                with open(path, "r", encoding="utf-8") as f:
                    mac = f.read().strip().upper()
                    if len(mac) == 17 and mac.count(":") == 5:
                        return mac
            except Exception:
                pass

    # 2. bluetoothctl list
    try:
        res = subprocess.run(["bluetoothctl", "list"], capture_output=True, text=True, timeout=2)
        if res.returncode == 0:
            for line in res.stdout.splitlines():
                m = re.search(r"Controller\s+([0-9A-Fa-f:]{17})", line)
                if m:
                    return m.group(1).upper()
    except Exception:
        pass

    # 3. btmgmt info
    try:
        res = subprocess.run(["btmgmt", "info"], capture_output=True, text=True, timeout=2)
        if res.returncode == 0:
            m = re.search(r"addr\s+([0-9A-Fa-f:]{17})", res.stdout)
            if m:
                return m.group(1).upper()
    except Exception:
        pass

    # 4. hciconfig
    try:
        res = subprocess.run(["hciconfig", "hci0"], capture_output=True, text=True, timeout=2)
        if res.returncode == 0:
            m = re.search(r"BD Address:\s*([0-9A-Fa-f:]{17})", res.stdout)
            if m:
                return m.group(1).upper()
    except Exception:
        pass

    return ""

def get_bluetooth_status():
    """Query Bluetooth adapter status and paired devices list with resolved names."""
    adapter_avail = os.path.exists("/sys/class/bluetooth/hci0")
    pi_mac = get_pi_bluetooth_mac()
    paired = []
    try:
        res = run_bluetoothctl(["paired-devices"], timeout=5)
        if res.returncode == 0:
            for line in res.stdout.strip().splitlines():
                m = re.match(r"^Device\s+([0-9A-Fa-f:]{17})\s*(.*)$", line.strip())
                if m:
                    mac = m.group(1).upper()
                    raw_name = m.group(2).strip()
                    name = resolve_device_name(mac, raw_name)
                    paired.append({"mac": mac, "name": name})
            adapter_avail = True
    except Exception as e:
        log_bt("BT", f"Error querying paired devices: {e}")

    return {
        "adapter_available": adapter_avail,
        "pi_mac": pi_mac,
        "pi_name": "ICarMeter",
        "paired_devices": paired,
        "rfcomm_listening": _rfcomm_listening
    }

def _bt_discovery_worker():
    global _bt_discovery_active, _scan_process
    log_bt("BT", "Starting comprehensive Bluetooth discovery across Classic BR/EDR and BLE...")

    # Ensure controller is unblocked, powered, and named ICarMeter
    try:
        subprocess.run(["rfkill", "unblock", "bluetooth"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "power", "on"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "system-alias", "ICarMeter"], capture_output=True, timeout=2)
    except Exception:
        pass

    # Start a persistent interactive bluetoothctl session
    # A persistent session guarantees BlueZ discovery NEVER stops unexpectedly
    p = None
    try:
        p = subprocess.Popen(
            ["bluetoothctl"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
        _scan_process = p
        # Configure discovery filter for all transports (Classic + LE) and start continuous scan
        init_cmds = (
            "menu scan\n"
            "clear\n"
            "transport auto\n"
            "duplicate-data on\n"
            "back\n"
            "discoverable on\n"
            "scan on\n"
        )
        p.stdin.write(init_cmds)
        p.stdin.flush()
    except Exception as e:
        log_bt("BT", f"Failed to start interactive bluetoothctl scan: {e}")

    # Reader thread for real-time [NEW] and [CHG] discovery events from bluetoothctl stdout
    def live_event_reader(proc):
        while _bt_discovery_active and proc and proc.poll() is None:
            try:
                line = proc.stdout.readline()
                if not line:
                    break
                line_str = line.strip()
                # Match [NEW] or [CHG] Device MAC [Name/details]
                m_dev = re.search(r"\[(NEW|CHG)\]\s+Device\s+([0-9A-Fa-f:]{17})\s*(.*)", line_str)
                if m_dev:
                    mac = m_dev.group(2).upper()
                    detail = m_dev.group(3).strip()

                    name_candidate = ""
                    if detail.startswith("Name:"):
                        name_candidate = detail.replace("Name:", "").strip()
                    elif detail.startswith("Alias:"):
                        name_candidate = detail.replace("Alias:", "").strip()
                    elif not detail.startswith("RSSI:") and not detail.startswith("Connected:") and not detail.startswith("TxPower:") and not detail.startswith("ManufacturerData"):
                        name_candidate = detail

                    # Resolve real friendly name
                    real_name = resolve_device_name(mac, name_candidate)

                    with _bt_discovery_lock:
                        prev = _bt_discovered_devices.get(mac)
                        if prev != real_name:
                            _bt_discovered_devices[mac] = real_name
                            log_bt("BT-Discovery", f"Found device: {real_name} ({mac})")
                            ws_broadcast({
                                "type": "bt_device_found",
                                "mac": mac,
                                "name": real_name,
                                "rssi": 0
                            })
                            bt_broadcast({
                                "type": "bt_device_found",
                                "mac": mac,
                                "name": real_name,
                                "rssi": 0
                            })
            except Exception:
                pass

    if p:
        reader_t = threading.Thread(target=live_event_reader, args=(p,), daemon=True)
        reader_t.start()

    # Background cycle: periodically sweep devices list and resolve names for cached devices
    while _bt_discovery_active:
        try:
            # 1. Sweep devices via bluetoothctl
            res = subprocess.run(["bluetoothctl", "devices"], capture_output=True, text=True, timeout=3)
            if res.returncode == 0 and res.stdout:
                for line in res.stdout.strip().splitlines():
                    m = re.match(r"^Device\s+([0-9A-Fa-f:]{17})\s*(.*)$", line.strip())
                    if m:
                        mac = m.group(1).upper()
                        raw_name = m.group(2).strip()
                        real_name = resolve_device_name(mac, raw_name)
                        with _bt_discovery_lock:
                            prev = _bt_discovered_devices.get(mac)
                            if prev != real_name:
                                _bt_discovered_devices[mac] = real_name
                                log_bt("BT-Discovery", f"Discovered device: {real_name} ({mac})")
                                ws_broadcast({
                                    "type": "bt_device_found",
                                    "mac": mac,
                                    "name": real_name,
                                    "rssi": 0
                                })
                                bt_broadcast({
                                    "type": "bt_device_found",
                                    "mac": mac,
                                    "name": real_name,
                                    "rssi": 0
                                })

            # 2. Check any devices that still have generic MAC names and resolve them
            with _bt_discovery_lock:
                devices_to_resolve = [
                    (m, n) for m, n in _bt_discovered_devices.items()
                    if is_generic_mac_name(n, m)
                ]
            for mac, _ in devices_to_resolve:
                if not _bt_discovery_active:
                    break
                fresh_name = resolve_device_name(mac)
                if not is_generic_mac_name(fresh_name, mac):
                    with _bt_discovery_lock:
                        _bt_discovered_devices[mac] = fresh_name
                    log_bt("BT-Discovery", f"Resolved device name: {fresh_name} ({mac})")
                    ws_broadcast({
                        "type": "bt_device_found",
                        "mac": mac,
                        "name": fresh_name,
                        "rssi": 0
                    })
                    bt_broadcast({
                        "type": "bt_device_found",
                        "mac": mac,
                        "name": fresh_name,
                        "rssi": 0
                    })
        except Exception:
            pass
        time.sleep(2.5)

    # Clean shutdown of interactive session
    if p:
        try:
            p.stdin.write("scan off\nquit\n")
            p.stdin.flush()
            p.terminate()
            p.wait(timeout=2)
        except Exception:
            try:
                p.kill()
            except Exception:
                pass
        _scan_process = None

def start_bluetooth_discovery():
    global _bt_discovery_active, _bt_discovered_devices
    with _bt_discovery_lock:
        if _bt_discovery_active:
            return
        _bt_discovery_active = True
        _bt_discovered_devices.clear()
        t = threading.Thread(target=_bt_discovery_worker, daemon=True)
        t.start()

def stop_bluetooth_discovery():
    global _bt_discovery_active, _scan_process
    with _bt_discovery_lock:
        _bt_discovery_active = False
    log_bt("BT", "Stopping Bluetooth scan...")
    if _scan_process:
        try:
            _scan_process.stdin.write("scan off\nquit\n")
            _scan_process.stdin.flush()
            _scan_process.terminate()
        except Exception:
            pass
        _scan_process = None
    try:
        subprocess.run(["bluetoothctl", "scan", "off"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "discoverable", "on"], capture_output=True, timeout=2)
    except Exception:
        pass

def pair_bluetooth_device(mac):
    log_bt("BT-Pair", f"Initiating pairing with {mac}...")
    try:
        if os.path.exists(PIN_MARKER):
            try:
                os.remove(PIN_MARKER)
            except Exception:
                pass

        # Make sure controller is discoverable and pairable
        subprocess.run(["bluetoothctl", "discoverable", "on"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "pairable", "on"], capture_output=True, timeout=2)

        # Pre-trust device
        run_bluetoothctl(["trust", mac], timeout=5)

        # Pair with 25s timeout
        res = run_bluetoothctl(["pair", mac], timeout=25)

        if os.path.exists(PIN_MARKER):
            try:
                os.remove(PIN_MARKER)
            except Exception:
                pass
        ws_broadcast({"type": "bt_pin_code", "pin": ""})
        bt_broadcast({"type": "bt_pin_code", "pin": ""})

        out_lower = ((res.stdout or "") + " " + (res.stderr or "")).lower()
        if res.returncode == 0 or "successful" in out_lower or "already paired" in out_lower:
            # Re-ensure trusted
            run_bluetoothctl(["trust", mac], timeout=3)
            log_bt("BT-Pair", f"Pairing with {mac} SUCCESSFUL")
            return True, "Paired successfully"
        err_msg = res.stderr.strip() or res.stdout.strip() or "Pairing failed"
        log_bt("BT-Pair", f"Pairing with {mac} FAILED: {err_msg}")
        return False, err_msg
    except subprocess.TimeoutExpired:
        ws_broadcast({"type": "bt_pin_code", "pin": ""})
        bt_broadcast({"type": "bt_pin_code", "pin": ""})
        log_bt("BT-Pair", f"Pairing with {mac} TIMED OUT")
        return False, "Pairing timed out"
    except Exception as e:
        ws_broadcast({"type": "bt_pin_code", "pin": ""})
        bt_broadcast({"type": "bt_pin_code", "pin": ""})
        log_bt("BT-Pair", f"Pairing with {mac} ERROR: {e}")
        return False, str(e)

def unpair_bluetooth_device(mac):
    log_bt("BT-Unpair", f"Removing paired device {mac}...")
    try:
        res = run_bluetoothctl(["remove", mac], timeout=5)
        success = (res.returncode == 0)
        log_bt("BT-Unpair", f"Remove {mac} result: {success}")
        return success
    except Exception as e:
        log_bt("BT-Unpair", f"Remove {mac} error: {e}")
        return False
    except Exception:
        return False

_last_ws_event = None
_last_ws_event_time = 0.0

def log_ws_event(event_name):
    """Writes a clean, short event name to /dev/shm/icarmeter_ws_event for dashboard slide-up display"""
    global _last_ws_event, _last_ws_event_time
    now = time.time()
    event_name = event_name.strip()

    # Priority hold: keep high-value events (Config Saved, Connected, Config Synced) visible for at least 2.5s before Telemetry replaces it
    if event_name == "Telemetry" and _last_ws_event in ("Connected", "Config Saved", "Config Synced"):
        if now - _last_ws_event_time < 2.5:
            return

    # Avoid redundant writes if event is unchanged and within 1s
    if _last_ws_event == event_name and (now - _last_ws_event_time < 1.0):
        return

    _last_ws_event = event_name
    _last_ws_event_time = now
    try:
        with open(WS_EVENT_MARKER, "w", encoding="utf-8") as f:
            f.write(event_name + "\n")
    except Exception:
        pass

WWW_ROOT = "/opt/icarmeter/www"
FALLBACK_WWW_PATHS = [
    "/home/pi/www",
    os.path.join(os.path.dirname(__file__), "www"),
    os.path.join(os.path.dirname(__file__), "../../../mobile-app/www"),
    "./www",
    "./mobile-app/www"
]

MIME_TYPES = {
    ".html": "text/html; charset=utf-8",
    ".htm": "text/html; charset=utf-8",
    ".js": "application/javascript; charset=utf-8",
    ".mjs": "application/javascript; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".json": "application/json; charset=utf-8",
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".gif": "image/gif",
    ".svg": "image/svg+xml",
    ".ico": "image/x-icon",
    ".txt": "text/plain; charset=utf-8",
    ".woff": "font/woff",
    ".woff2": "font/woff2",
    ".ttf": "font/ttf"
}

def touch_app_connected(name="Wi-Fi Mobile App"):
    try:
        with open(APP_CONN_MARKER, "w", encoding="utf-8") as f:
            f.write(name)
    except Exception:
        pass

def get_actual_config_path():
    return CONFIG_PATH

def get_actual_www_root():
    if os.path.isdir(WWW_ROOT):
        return WWW_ROOT
    for path in FALLBACK_WWW_PATHS:
        abs_p = os.path.abspath(path)
        if os.path.isdir(abs_p):
            return abs_p
    return WWW_ROOT


def read_config():
    """Reads /opt/icarmeter/config.toml into dict"""
    data = {
        "tpms": {
            "enabled": True,
            "adapter": "hci0",
            "fl": "A4:C1:38:00:00:01",
            "fr": "A4:C1:38:00:00:02",
            "rl": "A4:C1:38:00:00:03",
            "rr": "A4:C1:38:00:00:04",
            "alert": 2.2,
            "high_temp_alert": 70.0,
        },
        "gps": {
            "enabled": True,
            "device": "/dev/serial0",
            "baud_rate": 9600,
        },
        "display": {
            "framebuffer_device": "/dev/fb0",
            "target_fps": 60,
        },
    }

    path = CONFIG_PATH
    if not os.path.exists(path):
        return data

    try:
        section = None
        with open(path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line.startswith("#") or not line:
                    continue
                if line.startswith("[") and line.endswith("]"):
                    section = line[1:-1].strip()
                    continue
                if "=" in line:
                    k, v = [x.strip() for x in line.split("=", 1)]
                    v = v.split("#")[0].strip().strip('"').strip("'")
                    if section == "tpms":
                        if k == "enabled":
                            data["tpms"]["enabled"] = (v.lower() == "true")
                        elif k == "adapter":
                            data["tpms"]["adapter"] = v
                        elif k == "fl_sensor":
                            data["tpms"]["fl"] = v
                        elif k == "fr_sensor":
                            data["tpms"]["fr"] = v
                        elif k == "rl_sensor":
                            data["tpms"]["rl"] = v
                        elif k == "rr_sensor":
                            data["tpms"]["rr"] = v
                        elif k == "low_pressure_alert":
                            try:
                                data["tpms"]["alert"] = float(v)
                            except ValueError:
                                pass
                        elif k == "high_temp_alert":
                            try:
                                data["tpms"]["high_temp_alert"] = float(v)
                            except ValueError:
                                pass
                    elif section == "gps":
                        if k == "enabled":
                            data["gps"]["enabled"] = (v.lower() == "true")
                        elif k == "device":
                            data["gps"]["device"] = v
                        elif k == "baud_rate":
                            try:
                                data["gps"]["baud_rate"] = int(v)
                            except ValueError:
                                pass
                    elif section == "display":
                        if k == "framebuffer_device":
                            data["display"]["framebuffer_device"] = v
                        elif k == "target_fps":
                            try:
                                data["display"]["target_fps"] = int(v)
                            except ValueError:
                                pass
    except Exception as e:
        print(f"[API] Error reading config: {e}", file=sys.stderr)

    return data


def write_config_to_path(path, payload):
    lines = []
    if os.path.exists(path):
        try:
            with open(path, "r", encoding="utf-8") as f:
                lines = f.readlines()
        except Exception:
            lines = []

    if not lines:
        lines = [
            "# ==============================================================================\n",
            "# Car Digital Dashboard - Hardware Configuration for Raspberry Pi OS\n",
            "# ==============================================================================\n\n",
            "[gps]\n",
            'device = "/dev/serial0"\n',
            "baud_rate = 9600\n",
            "enabled = true\n\n",
            "[tpms]\n",
            "enabled = true\n",
            'adapter = "hci0"\n',
            'fl_sensor = "A4:C1:38:00:00:01"\n',
            'fr_sensor = "A4:C1:38:00:00:02"\n',
            'rl_sensor = "A4:C1:38:00:00:03"\n',
            'rr_sensor = "A4:C1:38:00:00:04"\n',
            "low_pressure_alert = 2.2\n",
            "high_temp_alert = 70.0\n\n",
            "[display]\n",
            'framebuffer_device = "/dev/fb0"\n',
            "target_fps = 60\n",
        ]

    new_lines = []
    in_section = None
    has_high_temp_line = any("high_temp_alert" in l for l in lines)
    updated_keys = set()

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            if in_section == "tpms":
                if "high_temp_alert" not in updated_keys and ("high_temp_alert" in payload or "temp_alert" in payload):
                    temp_val = float(payload.get("high_temp_alert", payload.get("temp_alert", 70.0)))
                    new_lines.append(f'high_temp_alert = {temp_val:.1f}\n')
                    updated_keys.add("high_temp_alert")
            in_section = stripped[1:-1].strip()
            new_lines.append(line)
            continue

        if in_section == "tpms" and "=" in line:
            k = line.split("=")[0].strip()
            updated_keys.add(k)
            if k == "fl_sensor" and "fl" in payload and payload["fl"]:
                line = f'fl_sensor = "{payload["fl"]}"\n'
            elif k == "fr_sensor" and "fr" in payload and payload["fr"]:
                line = f'fr_sensor = "{payload["fr"]}"\n'
            elif k == "rl_sensor" and "rl" in payload and payload["rl"]:
                line = f'rl_sensor = "{payload["rl"]}"\n'
            elif k == "rr_sensor" and "rr" in payload and payload["rr"]:
                line = f'rr_sensor = "{payload["rr"]}"\n'
            elif k == "low_pressure_alert":
                if "alert" in payload or "low_pressure_alert" in payload:
                    val = float(payload.get("alert", payload.get("low_pressure_alert", 2.2)))
                    line = f'low_pressure_alert = {val:.1f}\n'
                if not has_high_temp_line and ("high_temp_alert" in payload or "temp_alert" in payload):
                    temp_val = float(payload.get("high_temp_alert", payload.get("temp_alert", 70.0)))
                    line += f'high_temp_alert = {temp_val:.1f}\n'
                    updated_keys.add("high_temp_alert")
            elif k == "high_temp_alert" and ("high_temp_alert" in payload or "temp_alert" in payload):
                temp_val = float(payload.get("high_temp_alert", payload.get("temp_alert", 70.0)))
                line = f'high_temp_alert = {temp_val:.1f}\n'
            elif k == "enabled" and ("tpms_enabled" in payload or "enabled" in payload):
                val_bool = payload.get("tpms_enabled", payload.get("enabled"))
                val = "true" if val_bool else "false"
                line = f'enabled = {val}\n'
            elif k == "adapter" and "adapter" in payload and payload["adapter"]:
                line = f'adapter = "{payload["adapter"]}"\n'
        elif in_section == "gps" and "=" in line:
            k = line.split("=")[0].strip()
            updated_keys.add(k)
            if k == "enabled" and ("gps_enabled" in payload or "enabled" in payload):
                val_bool = payload.get("gps_enabled", payload.get("enabled"))
                val = "true" if val_bool else "false"
                line = f'enabled = {val}\n'
            elif k == "device" and "gps_device" in payload and payload["gps_device"]:
                line = f'device = "{payload["gps_device"]}"\n'
            elif k == "baud_rate" and "baud_rate" in payload:
                line = f'baud_rate = {int(payload["baud_rate"])}\n'
        elif in_section == "display" and "=" in line:
            k = line.split("=")[0].strip()
            updated_keys.add(k)
            if k == "target_fps" and "target_fps" in payload:
                line = f'target_fps = {int(payload["target_fps"])}\n'
            elif k == "framebuffer_device" and "framebuffer_device" in payload and payload["framebuffer_device"]:
                line = f'framebuffer_device = "{payload["framebuffer_device"]}"\n'

        new_lines.append(line)

    try:
        os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.writelines(new_lines)
            f.flush()
            try:
                os.fsync(f.fileno())
            except OSError:
                pass
        try:
            os.chmod(path, 0o666)
        except OSError:
            pass
        return True
    except Exception as e:
        print(f"[API] Error writing config to {path}: {e}", file=sys.stderr)
        return False


def write_config(payload):
    """Updates /opt/icarmeter/config.toml with payload, fsyncs and forces flush to non-volatile flash"""
    success = write_config_to_path(CONFIG_PATH, payload)

    # Force kernel dirty filesystem pages to write back to physical flash storage (SD card / eMMC)
    try:
        os.sync()
    except (AttributeError, OSError):
        pass

    touch_app_connected("Bluetooth Companion")
    print(f"[API] Synchronized, fsynced and flushed config to disk: {CONFIG_PATH}")
    if not success:
        raise IOError(f"Could not write configuration to {CONFIG_PATH}")


def handle_bt_command(msg_dict, reply_fn):
    """
    Dedicated command handler for all data & config operations.
    Exclusively permitted over Bluetooth RFCOMM/SPP.
    """
    touch_app_connected("Bluetooth Companion")
    msg_type = msg_dict.get("type", "")

    if msg_type == "get_bt_logs":
        with bt_logs_lock:
            logs_list = list(bt_logs_buffer)
        reply_fn({"type": "bt_logs", "logs": logs_list})
        return

    if msg_type == "clear_bt_logs":
        clear_bt_logs()
        reply_fn({"type": "bt_logs", "logs": []})
        return

    if msg_type not in ("ping", "telemetry", "gps"):
        log_bt("BT-RFCOMM", f"Received command: {msg_type}")

    if msg_type == "ping":
        reply_fn({"type": "pong", "timestamp": int(time.time())})
        return

    if msg_type == "get_config":
        log_ws_event("Config Synced")
        reply_fn({"type": "config", "data": read_config()})
        return

    if msg_type == "get_tpms":
        reply_fn({"type": "tpms", "data": read_tpms_telemetry()})
        return

    if msg_type == "set_config":
        payload = msg_dict.get("payload", {})
        try:
            write_config(payload)
            log_ws_event("Config Saved")
            # Broadcast updated config strictly to connected Bluetooth clients
            bt_broadcast({
                "type": "config",
                "data": read_config(),
                "message": "Saved",
                "status": "ok"
            })
        except Exception as e:
            reply_fn({"type": "error", "message": str(e)})
        return

    if msg_type in ("telemetry", "gps"):
        try:
            spd = float(msg_dict.get("spd", 0.0))
            lat = float(msg_dict.get("lat", 0.0))
            lon = float(msg_dict.get("lon", 0.0))
            alt = float(msg_dict.get("alt", 0.0))
            heading = float(msg_dict.get("heading", msg_dict.get("bearing", 0.0)))
            with open("/dev/shm/icarmeter_phone_telemetry", "w", encoding="utf-8") as f:
                f.write(f"spd={spd}\nlat={lat}\nlon={lon}\nalt={alt}\nheading={heading}\n")
            log_ws_event("Telemetry")
        except Exception:
            pass
        return

    if msg_type == "get_status":
        with bt_clients_lock:
            num_bt = len(connected_bt_clients)
        reply_fn({
            "type": "status",
            "data": {
                "status": "online",
                "app_connected": num_bt > 0,
                "clients": num_bt,
                "timestamp": int(time.time())
            }
        })
        return

    if msg_type == "get_bt_status":
        status_data = get_bluetooth_status()
        reply_fn({
            "type": "bt_status",
            "data": status_data,
            "pi_mac": status_data.get("pi_mac", ""),
            "pi_name": "ICarMeter"
        })
        return

    if msg_type == "bt_start_discovery":
        start_bluetooth_discovery()
        reply_fn({"type": "bt_discovery_started"})
        return

    if msg_type == "bt_stop_discovery":
        stop_bluetooth_discovery()
        reply_fn({"type": "bt_discovery_stopped"})
        return

    if msg_type == "bt_pair":
        target_mac = msg_dict.get("mac", "").strip().upper()
        def do_pair_bt_async():
            success, msg_text = pair_bluetooth_device(target_mac)
            bt_broadcast({
                "type": "bt_pair_result",
                "mac": target_mac,
                "success": success,
                "message": msg_text
            })
        threading.Thread(target=do_pair_bt_async, daemon=True).start()
        return

    if msg_type == "bt_unpair":
        target_mac = msg_dict.get("mac", "").strip().upper()
        success = unpair_bluetooth_device(target_mac)
        reply_fn({"type": "bt_unpair_result", "mac": target_mac, "success": success})
        return


def rfcomm_client_handler(client_sock, client_info):
    client_sock.settimeout(60.0)
    touch_app_connected("Bluetooth Companion")
    log_ws_event("Connected")
    with bt_clients_lock:
        connected_bt_clients.add(client_sock)
    log_bt("BT-RFCOMM", f"Client connected from {client_info}")
    print(f"[BT-RFCOMM] Client connected from {client_info}")

    welcome = {
        "type": "welcome",
        "status": "online",
        "app_connected": True,
        "timestamp": int(time.time()),
        "config": read_config()
    }
    try:
        client_sock.sendall((json.dumps(welcome) + "\n").encode("utf-8"))
    except Exception:
        with bt_clients_lock:
            connected_bt_clients.discard(client_sock)
        try:
            client_sock.close()
        except Exception:
            pass
        return

    buffer = ""
    try:
        while True:
            try:
                data = client_sock.recv(4096)
                if not data:
                    break
                buffer += data.decode("utf-8", errors="ignore")
                while "\n" in buffer:
                    line, buffer = buffer.split("\n", 1)
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        cmd = json.loads(line)
                        handle_bt_command(
                            cmd,
                            reply_fn=lambda r: client_sock.sendall((json.dumps(r) + "\n").encode("utf-8"))
                        )
                    except Exception as e:
                        print(f"[BT-RFCOMM] JSON error: {e}", file=sys.stderr)
            except socket.timeout:
                try:
                    client_sock.sendall((json.dumps({"type": "ping"}) + "\n").encode("utf-8"))
                except Exception:
                    break
    except Exception as e:
        print(f"[BT-RFCOMM] Connection error from {client_info}: {e}", file=sys.stderr)
    finally:
        with bt_clients_lock:
            connected_bt_clients.discard(client_sock)
        try:
            client_sock.close()
        except Exception:
            pass
        log_ws_event("Disconnected")
        log_bt("BT-RFCOMM", f"Client disconnected from {client_info}")
        print(f"[BT-RFCOMM] Client disconnected from {client_info}")


def rfcomm_server_worker():
    global _rfcomm_listening
    if not hasattr(socket, "AF_BLUETOOTH") or not hasattr(socket, "BTPROTO_RFCOMM"):
        log_bt("BT-RFCOMM", "AF_BLUETOOTH or BTPROTO_RFCOMM not supported in python socket.")
        print("[BT-RFCOMM] AF_BLUETOOTH or BTPROTO_RFCOMM not supported in python socket.", file=sys.stderr)
        return

    while True:
        server_sock = None
        try:
            ensure_bt_always_ready()
            try:
                subprocess.run(["sdptool", "add", "SP"], capture_output=True, timeout=3)
            except Exception:
                pass

            server_sock = socket.socket(socket.AF_BLUETOOTH, socket.SOCK_STREAM, socket.BTPROTO_RFCOMM)
            server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            server_sock.bind(("00:00:00:00:00:00", RFCOMM_PORT))
            server_sock.listen(4)
            _rfcomm_listening = True
            log_bt("BT-RFCOMM", f"RFCOMM server permanently listening on channel {RFCOMM_PORT}...")
            print(f"[BT-RFCOMM] RFCOMM server permanently listening on channel {RFCOMM_PORT}...")

            while True:
                client_sock, client_info = server_sock.accept()
                t = threading.Thread(
                    target=rfcomm_client_handler,
                    args=(client_sock, client_info),
                    daemon=True
                )
                t.start()
        except Exception as e:
            _rfcomm_listening = False
            if server_sock:
                try:
                    server_sock.close()
                except Exception:
                    pass
            log_bt("BT-RFCOMM", f"Server socket note: {e}. Re-establishing listening channel in 3s...")
            print(f"[BT-RFCOMM] Server socket note: {e}. Re-establishing listening channel in 3s...", file=sys.stderr)
            time.sleep(3)


class ApiHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def send_cors_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS, PUT, DELETE")
        self.send_header("Access-Control-Allow-Headers", "*")
        self.send_header("Access-Control-Max-Age", "86400")

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_cors_headers()
        self.send_header("Content-Length", "0")
        self.end_headers()

    def serve_static(self, path):
        www_dir = get_actual_www_root()
        clean_path = path.lstrip("/")
        if not clean_path or clean_path == "/":
            clean_path = "index.html"

        # Sanitize path to prevent directory traversal
        file_path = os.path.abspath(os.path.join(www_dir, clean_path))
        if not file_path.startswith(os.path.abspath(www_dir)):
            self.send_error(403, "Forbidden")
            return

        # If file doesn't exist or is a directory, fallback to index.html (SPA routing)
        if not os.path.exists(file_path) or os.path.isdir(file_path):
            file_path = os.path.join(www_dir, "index.html")

        if not os.path.exists(file_path):
            self.send_error(404, "Not Found")
            return

        ext = os.path.splitext(file_path)[1].lower()
        content_type = MIME_TYPES.get(ext, "application/octet-stream")

        try:
            with open(file_path, "rb") as f:
                content = f.read()
            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(content)))
            self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
            self.send_cors_headers()
            self.end_headers()
            self.wfile.write(content)
        except Exception as e:
            self.send_error(500, f"Error reading file: {e}")

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        # ─── WebSocket Upgrade Handling ───────────────────────────────────────
        if self.headers.get("Upgrade", "").lower() == "websocket" or path in ("/ws", "/ws/"):
            key = self.headers.get("Sec-WebSocket-Key")
            if not key:
                self.send_error(400, "Missing Sec-WebSocket-Key")
                return
            accept_token = base64.b64encode(hashlib.sha1((key.strip() + WS_MAGIC_GUID).encode("utf-8")).digest()).decode("utf-8")

            # Send 101 Switching Protocols
            self.send_response(101, "Switching Protocols")
            self.send_header("Upgrade", "websocket")
            self.send_header("Connection", "Upgrade")
            self.send_header("Sec-WebSocket-Accept", accept_token)
            self.end_headers()

            # Take over socket for WebSocket protocol on this worker thread
            self.handle_websocket()
            return

        # ─── Captive Portal Detection Endpoints ──────────────────────────────
        # Android generate_204 probe
        if path in ("/generate_204", "/gen_204"):
            self.send_response(204)
            self.send_cors_headers()
            self.send_header("Content-Length", "0")
            self.end_headers()
            return

        # Apple iOS hotspot-detect probe
        if path in ("/hotspot-detect.html", "/canonical.html", "/success.html", "/library/test/success.html"):
            body = b"<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>"
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.send_cors_headers()
            self.end_headers()
            self.wfile.write(body)
            return

        # Microsoft NCSI / Connect Test probes
        if path in ("/connecttest.txt",):
            body = b"Microsoft Connect Test"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.send_cors_headers()
            self.end_headers()
            self.wfile.write(body)
            return

        if path in ("/ncsi.txt",):
            body = b"Microsoft NCSI"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.send_cors_headers()
            self.end_headers()
            self.wfile.write(body)
            return

        # ─── Static Web Companion Files ──────────────────────────────────────
        self.serve_static(path)

    def handle_websocket(self):
        sock = self.connection
        sock.settimeout(60.0)
        with ws_clients_lock:
            connected_ws_clients.add(sock)

        client_info = f"{self.client_address[0]}:{self.client_address[1]}"
        print(f"[WS] Pairing setup client connected from {client_info}")

        pi_mac = get_pi_bluetooth_mac()
        welcome = {
            "type": "welcome",
            "status": "online",
            "pi_mac": pi_mac,
            "pi_name": "ICarMeter",
            "app_connected": False,
            "timestamp": int(time.time())
        }
        try:
            sock.sendall(encode_ws_frame(json.dumps(welcome).encode("utf-8"), 0x1))
        except Exception:
            with ws_clients_lock:
                connected_ws_clients.discard(sock)
            return

        try:
            while True:
                try:
                    opcode, payload = decode_ws_frame(sock)
                except socket.timeout:
                    # Timeout reached — send ping to check client liveness
                    try:
                        sock.sendall(encode_ws_frame(b"ping", opcode=0x9))
                        continue
                    except Exception:
                        break

                if opcode is None or opcode == 0x8:  # Socket closed or Close frame
                    try:
                        sock.sendall(encode_ws_frame(b"", opcode=0x8))
                    except Exception:
                        pass
                    break

                if opcode == 0x9:  # Ping from client
                    try:
                        sock.sendall(encode_ws_frame(payload, opcode=0xA))
                    except Exception:
                        break
                    continue

                if opcode == 0xA:  # Pong from client
                    continue

                if opcode == 0x1:  # Text message
                    text = payload.decode("utf-8", errors="ignore")
                    self.process_ws_message(sock, text)
        except Exception as e:
            print(f"[WS] Connection error from {client_info}: {e}", file=sys.stderr)
        finally:
            with ws_clients_lock:
                connected_ws_clients.discard(sock)
            log_ws_event("Disconnected")
            print(f"[WS] Client disconnected from {client_info}")

    def process_ws_message(self, sock, text):
        """
        Processes Wi-Fi WebSocket messages.
        Wi-Fi is strictly restricted to Bluetooth pairing/discovery setup.
        Zero config or data operations are processed here.
        """
        try:
            msg = json.loads(text)
        except Exception:
            return

        msg_type = msg.get("type", "")

        if msg_type == "ping":
            reply = {"type": "pong", "timestamp": int(time.time())}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "get_bt_status":
            status_data = get_bluetooth_status()
            reply = {
                "type": "bt_status",
                "data": status_data,
                "pi_mac": status_data.get("pi_mac", ""),
                "pi_name": "ICarMeter"
            }
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type in ("match_pi", "get_pi_info"):
            ensure_bt_always_ready()
            pi_mac = get_pi_bluetooth_mac()
            reply = {
                "type": "pi_matched",
                "pi_mac": pi_mac,
                "pi_name": "ICarMeter",
                "rfcomm_channel": RFCOMM_PORT,
                "wifi_ip": "192.168.4.1",
                "timestamp": int(time.time())
            }
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "bt_start_discovery":
            start_bluetooth_discovery()
            reply = {"type": "bt_discovery_started"}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "bt_stop_discovery":
            stop_bluetooth_discovery()
            reply = {"type": "bt_discovery_stopped"}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "bt_pair":
            target_mac = msg.get("mac", "").strip().upper()
            def do_pair_async():
                success, msg_text = pair_bluetooth_device(target_mac)
                ws_broadcast({
                    "type": "bt_pair_result",
                    "mac": target_mac,
                    "success": success,
                    "message": msg_text
                })
            threading.Thread(target=do_pair_async, daemon=True).start()
            return

        if msg_type == "bt_unpair":
            target_mac = msg.get("mac", "").strip().upper()
            success = unpair_bluetooth_device(target_mac)
            reply = {"type": "bt_unpair_result", "mac": target_mac, "success": success}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "get_bt_logs":
            with bt_logs_lock:
                logs_list = list(bt_logs_buffer)
            reply = {"type": "bt_logs", "logs": logs_list}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "clear_bt_logs":
            clear_bt_logs()
            reply = {"type": "bt_logs", "logs": []}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "get_status":
            with ws_clients_lock:
                num_clients = len(connected_ws_clients)
            reply = {
                "type": "status",
                "data": {
                    "status": "online",
                    "app_connected": False,
                    "clients": num_clients,
                    "timestamp": int(time.time())
                }
            }
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

    def do_POST(self):
        # All HTTP POST endpoints removed - data operations are strictly over Bluetooth
        self.send_error(404, "Not Found")

    def send_json(self, code, obj):
        body = json.dumps(obj).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_cors_headers()
        self.end_headers()
        self.wfile.write(body)


def start_http_listener(port):
    """Starts an HTTP server on the specified port. Runs in background thread."""
    try:
        httpd = http.server.ThreadingHTTPServer(("", port), ApiHandler)
        print(f"[API] ICarMeter HTTP listener active on port {port}")
        httpd.serve_forever()
    except Exception as e:
        print(f"[API] Note: port {port} listener not active ({e})", file=sys.stderr)


def main():
    # Initialize Bluetooth logs buffer from disk
    init_bt_logs_buffer()
    log_bt("SYSTEM", "ICarMeter API service initializing...")

    # Start bluetoothctl live event monitor worker
    t_btmon = threading.Thread(target=bluetoothctl_monitor_worker, daemon=True)
    t_btmon.start()

    # Start Bluetooth background heartbeat worker
    t_hb = threading.Thread(target=bt_heartbeat_worker, daemon=True)
    t_hb.start()

    # Start PIN/passkey monitor worker
    t_pin = threading.Thread(target=pin_monitor_worker, daemon=True)
    t_pin.start()

    # Start Bluetooth RFCOMM/SPP server worker
    t_rfcomm = threading.Thread(target=rfcomm_server_worker, daemon=True)
    t_rfcomm.start()

    # Start port 80 listener in background thread (standard web & captive portal)
    if HTTP_PORT != PORT:
        t80 = threading.Thread(target=start_http_listener, args=(HTTP_PORT,), daemon=True)
        t80.start()

    # Primary listener on PORT 8080 (serves WebSocket at /ws + HTTP Web Companion)
    server_address = ("", PORT)
    httpd = http.server.ThreadingHTTPServer(server_address, ApiHandler)
    print(f"[API] ICarMeter primary WebSocket (/ws) & Web server running on port {PORT}...")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()


if __name__ == "__main__":
    main()
