#!/usr/bin/env python3
"""
ICarMeter Bluetooth RFCOMM/SPP command service.
Handles vehicle configuration, telemetry, pairing, and log commands over Bluetooth.
"""
import collections
import datetime
import json
import os
import re
import socket
import subprocess
import sys
import threading
import time

try:
    import dbus
    import dbus.mainloop.glib
    from gi.repository import GLib
    HAS_DBUS = True
except ImportError:
    dbus = None
    GLib = None
    HAS_DBUS = False


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
    Dedicated worker streaming real-time events directly from D-Bus / bluetoothctl.
    Logs BlueZ adapter, device discovery, connection, pairing, and passkey events.
    """
    time.sleep(2)
    log_bt("SYSTEM", "Starting Bluetooth live event monitor...")

    # Primary: D-Bus signal monitoring with connection retry loop
    while HAS_DBUS and dbus and GLib:
        try:
            dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
            bus = dbus.SystemBus()

            def on_properties_changed(interface, changed, invalidated, path=None):
                if interface == "org.bluez.Device1":
                    mac = str(path).split("dev_")[-1].replace("_", ":").upper() if "dev_" in str(path) else str(path)
                    for k, v in changed.items():
                        if k in ("Connected", "Paired", "Name", "Alias", "RSSI"):
                            log_bt("bluetooth", f"Device {mac} {k}: {v}")
                        if k == "Connected":
                            status = "connected" if v else "disconnected"
                            bt_broadcast({"type": "bt_device_status", "mac": mac, "status": status})
                elif interface == "org.bluez.Adapter1":
                    for k, v in changed.items():
                        if k in ("Powered", "Discoverable", "Pairable"):
                            log_bt("bluetooth", f"Adapter {k}: {v}")

            bus.add_signal_receiver(
                on_properties_changed,
                signal_name="PropertiesChanged",
                dbus_interface="org.freedesktop.DBus.Properties",
                path_keyword="path"
            )
            log_bt("SYSTEM", "Active D-Bus Bluetooth signal monitor listening.")
            loop = GLib.MainLoop()
            loop.run()
            return
        except Exception as e:
            log_bt("SYSTEM", f"D-Bus monitor note ({e}). Retrying in 5s...")
            time.sleep(5)

    # Secondary Fallback: bluetoothctl streaming without EOF
    log_bt("SYSTEM", "D-Bus unavailable, starting passive bluetoothctl stream...")
    ignore_patterns = (
        "Waiting to connect to bluetoothd...",
        "quit",
        "Agent registered",
        "[bluetooth]#",
    )
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
                clean = re.sub(r'\[bluetooth\s*\]#\s*', '', clean).strip()
                clean_line = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', clean).strip()
                if not clean_line or any(clean_line.startswith(ig) for ig in ignore_patterns) or clean_line == "quit":
                    continue
                log_bt("bluetoothctl", clean_line)
            p.wait()
        except Exception as e:
            log_bt("bluetoothctl", f"Monitor stream note: {e}")
        time.sleep(5)

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
        # Check and configure via D-Bus first to avoid spawning subprocesses
        try:
            if not dbus:
                raise RuntimeError("D-Bus not available")
            bus = dbus.SystemBus()
            adapter_obj = bus.get_object("org.bluez", "/org/bluez/hci0")
            props = dbus.Interface(adapter_obj, "org.freedesktop.DBus.Properties")

            powered = bool(props.Get("org.bluez.Adapter1", "Powered"))
            discoverable = bool(props.Get("org.bluez.Adapter1", "Discoverable"))
            pairable = bool(props.Get("org.bluez.Adapter1", "Pairable"))
            alias = str(props.Get("org.bluez.Adapter1", "Alias"))

            if not powered:
                props.Set("org.bluez.Adapter1", "Powered", dbus.Boolean(True))
            if alias != "ICarMeter":
                props.Set("org.bluez.Adapter1", "Alias", dbus.String("ICarMeter"))
            if not discoverable:
                props.Set("org.bluez.Adapter1", "DiscoverableTimeout", dbus.UInt32(0))
                props.Set("org.bluez.Adapter1", "Discoverable", dbus.Boolean(True))
            if not pairable:
                props.Set("org.bluez.Adapter1", "PairableTimeout", dbus.UInt32(0))
                props.Set("org.bluez.Adapter1", "Pairable", dbus.Boolean(True))
            return
        except Exception:
            pass

        # Fallback to btmgmt or bluetoothctl
        if os.path.exists("/usr/bin/btmgmt"):
            subprocess.run(["btmgmt", "--index", "0", "power", "on"], capture_output=True, timeout=2)
            subprocess.run(["btmgmt", "--index", "0", "name", "ICarMeter", "ICarMeter"], capture_output=True, timeout=2)
            subprocess.run(["btmgmt", "--index", "0", "discov", "on"], capture_output=True, timeout=2)
            subprocess.run(["btmgmt", "--index", "0", "pairable", "on"], capture_output=True, timeout=2)
        else:
            subprocess.run(["bluetoothctl", "power", "on"], capture_output=True, timeout=2)
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
            touch_app_connected()
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
        else:
            clear_app_connected()

        # Periodically ensure adapter never goes into sleep/non-discoverable mode
        if tick % 30 == 0:
            ensure_bt_always_ready()

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
PIN_MARKER = "/dev/shm/icarmeter_bt_pin"
TPMS_TELEMETRY_MARKER = "/dev/shm/icarmeter_tpms_telemetry"
APP_CONNECTED_MARKER = "/dev/shm/icarmeter_app_connected"

def touch_app_connected(name="Phone Connected"):
    try:
        with open(APP_CONNECTED_MARKER, "w", encoding="utf-8") as f:
            f.write(str(name).strip() if name else "Phone Connected")
    except Exception:
        pass

def clear_app_connected():
    try:
        if os.path.exists(APP_CONNECTED_MARKER):
            os.remove(APP_CONNECTED_MARKER)
    except Exception:
        pass

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
    """Watches /dev/shm/icarmeter_bt_pin and broadcasts bt_pin_code over RFCOMM."""
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
                                bt_broadcast({
                                    "type": "bt_pin_code",
                                    "pin": pin,
                                    "mac": mac,
                                    "name": "ICarMeter"
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
    if re.match(r"^[0-9A-F]{12}$", clean_n):
        return True
    return False

def get_dbus_device_info(mac):
    """Retrieves device properties via BlueZ D-Bus interface."""
    if not HAS_DBUS:
        return None
    try:
        bus = dbus.SystemBus()
        om = dbus.Interface(bus.get_object("org.bluez", "/"), "org.freedesktop.DBus.ObjectManager")
        objects = om.GetManagedObjects()
        for path, interfaces in objects.items():
            if "org.bluez.Device1" in interfaces:
                props = interfaces["org.bluez.Device1"]
                addr = str(props.get("Address", "")).upper()
                if addr == mac:
                    alias = str(props.get("Alias", "")).strip()
                    name = str(props.get("Name", "")).strip()
                    paired = bool(props.get("Paired", False))
                    trusted = bool(props.get("Trusted", False))
                    connected = bool(props.get("Connected", False))
                    return {
                        "name": alias or name,
                        "alias": alias,
                        "raw_name": name,
                        "paired": paired,
                        "trusted": trusted,
                        "connected": connected,
                    }
    except Exception:
        pass
    return None

def get_bluetoothctl_device_info(mac):
    """Extracts device information via `bluetoothctl info <mac>`."""
    try:
        res = subprocess.run(["bluetoothctl", "info", mac], capture_output=True, text=True, timeout=3)
        if res.returncode == 0 and res.stdout:
            info = {}
            for line in res.stdout.splitlines():
                line = line.strip()
                if line.startswith("Name:"):
                    info["name"] = line.replace("Name:", "").strip()
                elif line.startswith("Alias:"):
                    info["alias"] = line.replace("Alias:", "").strip()
                elif line.startswith("Paired:"):
                    info["paired"] = ("yes" in line.lower())
                elif line.startswith("Trusted:"):
                    info["trusted"] = ("yes" in line.lower())
                elif line.startswith("Connected:"):
                    info["connected"] = ("yes" in line.lower())
            resolved = info.get("alias") or info.get("name") or ""
            if resolved:
                info["name"] = resolved
            return info
    except Exception:
        pass
    return None

def resolve_device_name(mac, initial_name=""):
    """
    Attempts to resolve friendly Bluetooth name via D-Bus and bluetoothctl info.
    Falls back to a sanitized fallback if no name has been broadcasted yet.
    """
    clean_init = str(initial_name).strip() if initial_name else ""
    if clean_init and not is_generic_mac_name(clean_init, mac):
        return clean_init

    # 1. Try D-Bus query (fastest & most direct)
    dbus_info = get_dbus_device_info(mac)
    if dbus_info and dbus_info.get("name") and not is_generic_mac_name(dbus_info["name"], mac):
        return dbus_info["name"]

    # 2. Try bluetoothctl info
    ctl_info = get_bluetoothctl_device_info(mac)
    if ctl_info and ctl_info.get("name") and not is_generic_mac_name(ctl_info["name"], mac):
        return ctl_info["name"]

    # 3. Clean fallback
    if clean_init:
        return clean_init
    suffix = mac[-5:].replace(":", "") if len(mac) >= 5 else mac
    return f"Thiết bị ({suffix})"

def get_pi_bluetooth_mac():
    """Extracts Raspberry Pi's Bluetooth controller MAC address."""
    for sys_path in ("/sys/class/bluetooth/hci0/address", "/sys/class/bluetooth/hci1/address"):
        try:
            if os.path.exists(sys_path):
                with open(sys_path, "r", encoding="utf-8") as f:
                    content = f.read().strip()
                if len(content) == 17 and ":" in content:
                    return content.upper()
        except Exception:
            pass

    try:
        res = subprocess.run(["bluetoothctl", "show"], capture_output=True, text=True, timeout=3)
        if res.returncode == 0 and res.stdout:
            for line in res.stdout.splitlines():
                m = re.search(r"Controller\s+([0-9A-Fa-f:]{17})", line)
                if m:
                    return m.group(1).upper()
    except Exception:
        pass
    return ""

def get_bluetooth_status():
    """Returns status of Bluetooth controller, paired devices, and server states."""
    pi_mac = get_pi_bluetooth_mac()
    paired = []
    try:
        res = subprocess.run(["bluetoothctl", "paired-devices"], capture_output=True, text=True, timeout=3)
        if res.returncode == 0 and res.stdout:
            for line in res.stdout.strip().splitlines():
                m = re.match(r"^Device\s+([0-9A-Fa-f:]{17})\s*(.*)$", line.strip())
                if m:
                    mac = m.group(1).upper()
                    raw_name = m.group(2).strip()
                    name = resolve_device_name(mac, raw_name)
                    paired.append({
                        "mac": mac,
                        "name": name
                    })
    except Exception as e:
        log_bt("BT", f"Error listing paired devices: {e}")

    return {
        "pi_mac": pi_mac,
        "pi_name": "ICarMeter",
        "paired_devices": paired,
        "rfcomm_listening": _rfcomm_listening
    }

def _bt_discovery_worker():
    global _bt_discovery_active, _scan_process
    log_bt("BT", "Starting comprehensive Bluetooth discovery across Classic BR/EDR and BLE...")

    # Ensure controller is unblocked and powered
    try:
        subprocess.run(["rfkill", "unblock", "bluetooth"], capture_output=True, timeout=2)
        ensure_bt_always_ready()
    except Exception:
        pass

    dbus_discovery_used = False
    try:
        if not dbus:
            raise RuntimeError("D-Bus not available")
        bus = dbus.SystemBus()
        adapter_obj = bus.get_object("org.bluez", "/org/bluez/hci0")
        adapter = dbus.Interface(adapter_obj, "org.bluez.Adapter1")
        adapter.StartDiscovery()
        dbus_discovery_used = True
        log_bt("BT", "Bluetooth discovery active via BlueZ D-Bus Adapter1.")
    except Exception as e:
        log_bt("BT", f"D-Bus StartDiscovery note: {e}, falling back to bluetoothctl...")

    p = None
    if not dbus_discovery_used:
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
            init_cmds = "scan on\n"
            p.stdin.write(init_cmds)
            p.stdin.flush()
        except Exception as e:
            log_bt("BT", f"Failed to start fallback bluetoothctl scan: {e}")

    # Reader thread for real-time [NEW] and [CHG] discovery events from bluetoothctl stdout (fallback)
    def live_event_reader(proc):
        while _bt_discovery_active and proc and proc.poll() is None:
            try:
                line = proc.stdout.readline()
                if not line:
                    break
                line_str = line.strip()
                m_dev = re.search(r"\[(NEW|CHG)\]\s+Device\s+([0-9A-Fa-f:]{17})\s*(.*)", line_str)
                if m_dev:
                    mac = m_dev.group(2).upper()
                    detail = m_dev.group(3).strip()

                    name_candidate = ""
                    if detail.startswith("Name:"):
                        name_candidate = detail.replace("Name:", "").strip()
                    elif detail.startswith("Alias:"):
                        name_candidate = detail.replace("Alias:", "").strip()
                    elif not any(detail.startswith(x) for x in ("RSSI:", "Connected:", "TxPower:", "ManufacturerData")):
                        name_candidate = detail

                    real_name = resolve_device_name(mac, name_candidate)
                    with _bt_discovery_lock:
                        prev = _bt_discovered_devices.get(mac)
                        if prev != real_name:
                            _bt_discovered_devices[mac] = real_name
                            log_bt("BT-Discovery", f"Found device: {real_name} ({mac})")
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
            if dbus_discovery_used and HAS_DBUS:
                try:
                    bus = dbus.SystemBus()
                    manager = dbus.Interface(bus.get_object("org.bluez", "/"), "org.freedesktop.DBus.ObjectManager")
                    objects = manager.GetManagedObjects()
                    for path, interfaces in objects.items():
                        if "org.bluez.Device1" in interfaces:
                            dev = interfaces["org.bluez.Device1"]
                            mac = str(dev.get("Address", "")).upper()
                            if mac:
                                alias = str(dev.get("Alias", dev.get("Name", "")))
                                real_name = resolve_device_name(mac, alias)
                                with _bt_discovery_lock:
                                    prev = _bt_discovered_devices.get(mac)
                                    if prev != real_name:
                                        _bt_discovered_devices[mac] = real_name
                                        log_bt("BT-Discovery", f"Discovered device: {real_name} ({mac})")
                                        bt_broadcast({
                                            "type": "bt_device_found",
                                            "mac": mac,
                                            "name": real_name,
                                            "rssi": 0
                                        })
                except Exception as e:
                    log_bt("BT-Discovery", f"D-Bus sweep note: {e}")
            else:
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
                    bt_broadcast({
                        "type": "bt_device_found",
                        "mac": mac,
                        "name": fresh_name,
                        "rssi": 0
                    })
        except Exception:
            pass
        time.sleep(2.5)

    # Clean shutdown
    if dbus_discovery_used and HAS_DBUS:
        try:
            bus = dbus.SystemBus()
            adapter_obj = bus.get_object("org.bluez", "/org/bluez/hci0")
            adapter = dbus.Interface(adapter_obj, "org.bluez.Adapter1")
            adapter.StopDiscovery()
            log_bt("BT", "Bluetooth discovery stopped via D-Bus.")
        except Exception:
            pass
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
    if dbus:
        try:
            bus = dbus.SystemBus()
            adapter_obj = bus.get_object("org.bluez", "/org/bluez/hci0")
            adapter = dbus.Interface(adapter_obj, "org.bluez.Adapter1")
            adapter.StopDiscovery()
        except Exception:
            pass
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
        bt_broadcast({"type": "bt_pin_code", "pin": ""})
        log_bt("BT-Pair", f"Pairing with {mac} TIMED OUT")
        return False, "Pairing timed out"
    except Exception as e:
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

def get_actual_config_path():
    return CONFIG_PATH


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
    print(f"[API] Synchronized, fsynced and flushed config to disk: {CONFIG_PATH}")
    if not success:
        raise IOError(f"Could not write configuration to {CONFIG_PATH}")


def handle_bt_command(msg_dict, reply_fn):
    """
    Dedicated command handler for all data & config operations.
    Exclusively permitted over Bluetooth RFCOMM/SPP.
    """
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
        reply_fn({"type": "config", "data": read_config()})
        return

    if msg_type == "get_tpms":
        reply_fn({"type": "tpms", "data": read_tpms_telemetry()})
        return

    if msg_type == "set_config":
        payload = msg_dict.get("payload", {})
        try:
            write_config(payload)
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

    if msg_type in ("match_pi", "get_pi_info"):
        ensure_bt_always_ready()
        pi_mac = get_pi_bluetooth_mac()
        reply_fn({
            "type": "pi_matched",
            "pi_mac": pi_mac,
            "pi_name": "ICarMeter",
            "rfcomm_channel": RFCOMM_PORT,
            "timestamp": int(time.time())
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
    with bt_clients_lock:
        connected_bt_clients.add(client_sock)
    peer_name = client_info[0] if isinstance(client_info, (list, tuple)) and client_info else "Phone Connected"
    touch_app_connected(peer_name)
    log_bt("BT-RFCOMM", f"Client connected from {client_info}")
    print(f"[BT-RFCOMM] Client connected from {client_info}")

    welcome = {
        "type": "welcome",
        "status": "online",
        "app_connected": True,
        "timestamp": int(time.time()),
        "config": read_config(),
        "pi_mac": get_pi_bluetooth_mac(),
        "pi_name": "ICarMeter"
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
            if not connected_bt_clients:
                clear_app_connected()
        try:
            client_sock.close()
        except Exception:
            pass
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
                for sdp_path in ("/var/run/sdp", "/run/sdp"):
                    if os.path.exists(sdp_path):
                        try:
                            os.chmod(sdp_path, 0o777)
                        except Exception:
                            pass
                res = subprocess.run(["sdptool", "add", f"--channel={RFCOMM_PORT}", "SP"], capture_output=True, timeout=3)
                if res.returncode != 0:
                    subprocess.run(["sdptool", "add", "SP"], capture_output=True, timeout=3)
                log_bt("BT-RFCOMM", f"SDP SPP profile verified on RFCOMM channel {RFCOMM_PORT}")
            except Exception as e:
                log_bt("BT-RFCOMM", f"Note registering SDP SPP: {e}")

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


def main():
    init_bt_logs_buffer()
    log_bt("SYSTEM", "ICarMeter Bluetooth RFCOMM service initializing...")

    threading.Thread(target=bluetoothctl_monitor_worker, daemon=True).start()
    threading.Thread(target=bt_heartbeat_worker, daemon=True).start()
    threading.Thread(target=pin_monitor_worker, daemon=True).start()

    # Keep the foreground service alive on the Bluetooth RFCOMM listener.
    rfcomm_server_worker()


if __name__ == "__main__":
    main()
