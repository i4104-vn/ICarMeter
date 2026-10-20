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
    """Keeps Bluetooth adapter powered, discoverable, and pairable at all times."""
    try:
        subprocess.run(["bluetoothctl", "power", "on"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "discoverable-timeout", "0"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "pairable-timeout", "0"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "discoverable", "on"], capture_output=True, timeout=2)
        subprocess.run(["bluetoothctl", "pairable", "on"], capture_output=True, timeout=2)
    except Exception:
        pass

def bt_heartbeat_worker():
    """Background worker: keeps app connection marker fresh when Bluetooth clients are active and ensures channel stays open."""
    tick = 0
    while True:
        time.sleep(4)
        tick += 1
        with bt_clients_lock:
            num_bt = len(connected_bt_clients)
        if num_bt > 0:
            touch_app_connected("Bluetooth")
            bt_broadcast({
                "type": "heartbeat",
                "status": "online",
                "clients": num_bt,
                "timestamp": int(time.time())
            })

        # Periodically ensure adapter never goes into sleep/non-discoverable mode
        if tick % 5 == 0:
            ensure_bt_always_ready()

PORT = 8080
HTTP_PORT = 80
CONFIG_PATH = "/opt/icarmeter/config.toml"
APP_CONN_MARKER = "/dev/shm/icarmeter_app_connected"
WS_EVENT_MARKER = "/dev/shm/icarmeter_ws_event"
PIN_MARKER = "/dev/shm/icarmeter_bt_pin"

_rfcomm_listening = False
_bt_discovery_active = False
_bt_discovery_lock = threading.Lock()
_bt_discovered_devices = set()
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

def get_bluetooth_status():
    """Query Bluetooth adapter status and paired devices list."""
    adapter_avail = os.path.exists("/sys/class/bluetooth/hci0")
    paired = []
    try:
        res = run_bluetoothctl(["paired-devices"], timeout=5)
        if res.returncode == 0:
            for line in res.stdout.strip().splitlines():
                m = re.match(r"^Device\s+([0-9A-Fa-f:]{17})\s*(.*)$", line.strip())
                if m:
                    mac = m.group(1).upper()
                    name = m.group(2).strip() or mac
                    paired.append({"mac": mac, "name": name})
            adapter_avail = True
    except Exception as e:
        log_bt("BT", f"Error querying paired devices: {e}")

    return {
        "adapter_available": adapter_avail,
        "paired_devices": paired,
        "rfcomm_listening": _rfcomm_listening
    }

def _bt_discovery_worker():
    global _bt_discovery_active
    log_bt("BT", "Starting Bluetooth scan & discovery...")
    try:
        run_bluetoothctl(["discoverable", "on"], timeout=3)
        run_bluetoothctl(["scan", "on"], timeout=3)
    except Exception:
        pass

    while _bt_discovery_active:
        try:
            res = run_bluetoothctl(["devices"], timeout=4)
            if res.returncode == 0:
                for line in res.stdout.strip().splitlines():
                    m = re.match(r"^Device\s+([0-9A-Fa-f:]{17})\s*(.*)$", line.strip())
                    if m:
                        mac = m.group(1).upper()
                        name = m.group(2).strip() or mac
                        with _bt_discovery_lock:
                            if mac not in _bt_discovered_devices:
                                _bt_discovered_devices.add(mac)
                                log_bt("BT-Discovery", f"Found device: {name} ({mac})")
                                ws_broadcast({
                                    "type": "bt_device_found",
                                    "mac": mac,
                                    "name": name,
                                    "rssi": 0
                                })
        except Exception:
            pass
        time.sleep(2)

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
    global _bt_discovery_active
    with _bt_discovery_lock:
        _bt_discovery_active = False
    log_bt("BT", "Stopping Bluetooth scan...")
    try:
        run_bluetoothctl(["scan", "off"], timeout=3)
        run_bluetoothctl(["discoverable", "on"], timeout=3)
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

        run_bluetoothctl(["trust", mac], timeout=5)
        res = run_bluetoothctl(["pair", mac], timeout=30)

        if os.path.exists(PIN_MARKER):
            try:
                os.remove(PIN_MARKER)
            except Exception:
                pass
        ws_broadcast({"type": "bt_pin_code", "pin": ""})

        if res.returncode == 0 or "successful" in res.stdout.lower():
            log_bt("BT-Pair", f"Pairing with {mac} SUCCESSFUL")
            return True, "Paired successfully"
        err_msg = res.stderr.strip() or res.stdout.strip() or "Pairing failed"
        log_bt("BT-Pair", f"Pairing with {mac} FAILED: {err_msg}")
        return False, err_msg
    except subprocess.TimeoutExpired:
        ws_broadcast({"type": "bt_pin_code", "pin": ""})
        log_bt("BT-Pair", f"Pairing with {mac} TIMED OUT")
        return False, "Pairing timed out"
    except Exception as e:
        ws_broadcast({"type": "bt_pin_code", "pin": ""})
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
            elif k == "enabled" and "tpms_enabled" in payload:
                val = "true" if payload["tpms_enabled"] else "false"
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
            with open("/dev/shm/icarmeter_phone_telemetry", "w", encoding="utf-8") as f:
                f.write(f"spd={spd}\nlat={lat}\nlon={lon}\nalt={alt}\n")
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

        welcome = {
            "type": "welcome",
            "status": "online",
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
            reply = {"type": "bt_status", "data": status_data}
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
