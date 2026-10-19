#!/usr/bin/env python3
"""
ICarMeter Headless Wi-Fi Configuration API Service
Listens for JSON API requests from the ICarMeter Mobile App over Wi-Fi Hotspot (192.168.4.1:8080)
to read, write, and synchronize /opt/icarmeter/config.toml in real time.
"""

import base64
import hashlib
import http.server
import json
import os
import socket
import struct
import sys
import threading
import time
import urllib.parse

WS_MAGIC_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"

# Thread-safe WebSocket client registry
connected_ws_clients = set()
ws_clients_lock = threading.Lock()

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

def ws_heartbeat_worker():
    """Background worker: keeps app connection marker fresh when clients are active."""
    while True:
        time.sleep(4)
        with ws_clients_lock:
            num = len(connected_ws_clients)
        if num > 0:
            touch_app_connected("WebSocket Companion")
            ws_broadcast({
                "type": "heartbeat",
                "status": "online",
                "clients": num,
                "timestamp": int(time.time())
            })

PORT = 8080
HTTP_PORT = 80
CONFIG_PATH = "/opt/icarmeter/config.toml"
FALLBACK_CONFIG_PATHS = ["/home/pi/config.toml", "./config.toml"]
APP_CONN_MARKER = "/dev/shm/icarmeter_app_connected"
WS_EVENT_MARKER = "/dev/shm/icarmeter_ws_event"

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
    if os.path.exists(CONFIG_PATH):
        return CONFIG_PATH
    for path in FALLBACK_CONFIG_PATHS:
        if os.path.exists(path):
            return path
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

    path = get_actual_config_path()
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


def write_config(payload):
    """Updates config.toml with payload and preserves file layout"""
    path = get_actual_config_path()
    lines = []
    if os.path.exists(path):
        with open(path, "r", encoding="utf-8") as f:
            lines = f.readlines()
    else:
        lines = [
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

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_section = stripped[1:-1].strip()
            new_lines.append(line)
            continue

        if in_section == "tpms" and "=" in line:
            k = line.split("=")[0].strip()
            if k == "fl_sensor" and "fl" in payload:
                line = f'fl_sensor = "{payload["fl"]}"\n'
            elif k == "fr_sensor" and "fr" in payload:
                line = f'fr_sensor = "{payload["fr"]}"\n'
            elif k == "rl_sensor" and "rl" in payload:
                line = f'rl_sensor = "{payload["rl"]}"\n'
            elif k == "rr_sensor" and "rr" in payload:
                line = f'rr_sensor = "{payload["rr"]}"\n'
            elif k == "low_pressure_alert":
                if "alert" in payload:
                    line = f'low_pressure_alert = {float(payload["alert"]):.1f}\n'
                if not has_high_temp_line and ("high_temp_alert" in payload or "temp_alert" in payload):
                    temp_val = float(payload.get("high_temp_alert", payload.get("temp_alert", 70.0)))
                    line += f'high_temp_alert = {temp_val:.1f}\n'
            elif k == "high_temp_alert" and ("high_temp_alert" in payload or "temp_alert" in payload):
                temp_val = float(payload.get("high_temp_alert", payload.get("temp_alert", 70.0)))
                line = f'high_temp_alert = {temp_val:.1f}\n'
            elif k == "enabled" and "tpms_enabled" in payload:
                val = "true" if payload["tpms_enabled"] else "false"
                line = f'enabled = {val}\n'
            elif k == "adapter" and "adapter" in payload:
                line = f'adapter = "{payload["adapter"]}"\n'
        elif in_section == "gps" and "=" in line:
            k = line.split("=")[0].strip()
            if k == "enabled" and ("gps_enabled" in payload or "enabled" in payload):
                val_bool = payload.get("gps_enabled", payload.get("enabled"))
                val = "true" if val_bool else "false"
                line = f'enabled = {val}\n'
            elif k == "device" and "gps_device" in payload:
                line = f'device = "{payload["gps_device"]}"\n'
            elif k == "baud_rate" and "baud_rate" in payload:
                line = f'baud_rate = {int(payload["baud_rate"])}\n'
        elif in_section == "display" and "=" in line:
            k = line.split("=")[0].strip()
            if k == "target_fps" and "target_fps" in payload:
                line = f'target_fps = {int(payload["target_fps"])}\n'
            elif k == "framebuffer_device" and "framebuffer_device" in payload:
                line = f'framebuffer_device = "{payload["framebuffer_device"]}"\n'

        new_lines.append(line)

    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(new_lines)

    touch_app_connected("Wi-Fi Mobile App")
    print(f"[API] Updated config in {path}")


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

        # ─── API Routes (Fallback & Diagnostic) ──────────────────────────────
        if path == "/api/config":
            touch_app_connected("Wi-Fi Mobile App")
            data = read_config()
            self.send_json(200, data)
            return

        if path == "/api/status":
            touch_app_connected("Wi-Fi Mobile App")
            with ws_clients_lock:
                num_clients = len(connected_ws_clients)
            self.send_json(200, {
                "status": "online",
                "app_connected": True,
                "ws_clients": num_clients,
                "timestamp": int(time.time()),
            })
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
        touch_app_connected("WebSocket Companion")
        log_ws_event("Connected")
        with ws_clients_lock:
            connected_ws_clients.add(sock)

        client_info = f"{self.client_address[0]}:{self.client_address[1]}"
        print(f"[WS] Client connected from {client_info}")

        # Send welcome payload with full configuration and status
        welcome = {
            "type": "welcome",
            "status": "online",
            "app_connected": True,
            "timestamp": int(time.time()),
            "config": read_config()
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
        touch_app_connected("WebSocket Companion")
        try:
            msg = json.loads(text)
        except Exception:
            return

        msg_type = msg.get("type", "")

        if msg_type == "ping":
            reply = {"type": "pong", "timestamp": int(time.time())}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "get_config":
            log_ws_event("Config Synced")
            reply = {"type": "config", "data": read_config()}
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

        if msg_type == "set_config":
            payload = msg.get("payload", {})
            try:
                write_config(payload)
                log_ws_event("Config Saved")
                # Broadcast updated config to ALL connected clients so every open UI syncs
                ws_broadcast({
                    "type": "config",
                    "data": read_config(),
                    "message": "Saved",
                    "status": "ok"
                })
            except Exception as e:
                err_reply = {"type": "error", "message": str(e)}
                sock.sendall(encode_ws_frame(json.dumps(err_reply).encode("utf-8"), 0x1))
            return

        if msg_type in ("telemetry", "gps"):
            try:
                spd = float(msg.get("spd", 0.0))
                lat = float(msg.get("lat", 0.0))
                lon = float(msg.get("lon", 0.0))
                alt = float(msg.get("alt", 0.0))
                with open("/dev/shm/icarmeter_phone_telemetry", "w", encoding="utf-8") as f:
                    f.write(f"spd={spd}\nlat={lat}\nlon={lon}\nalt={alt}\n")
                log_ws_event("Telemetry")
            except Exception:
                pass
            return

        if msg_type == "get_status":
            with ws_clients_lock:
                num_clients = len(connected_ws_clients)
            reply = {
                "type": "status",
                "data": {
                    "status": "online",
                    "app_connected": True,
                    "clients": num_clients,
                    "timestamp": int(time.time())
                }
            }
            sock.sendall(encode_ws_frame(json.dumps(reply).encode("utf-8"), 0x1))
            return

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        content_length = int(self.headers.get("Content-Length", 0))
        post_body = self.rfile.read(content_length)

        if path == "/api/config":
            try:
                payload = json.loads(post_body.decode("utf-8"))
                write_config(payload)
                log_ws_event("Config Saved")
                # Broadcast updated config to all WebSocket clients
                ws_broadcast({
                    "type": "config",
                    "data": read_config(),
                    "message": "Saved",
                    "status": "ok"
                })
                self.send_json(200, {"status": "ok", "message": "Saved"})
            except Exception as e:
                self.send_json(400, {"status": "error", "message": str(e)})
            return

        if path in ("/api/gps", "/api/telemetry"):
            try:
                payload = json.loads(post_body.decode("utf-8"))
                spd = float(payload.get("spd", 0.0))
                lat = float(payload.get("lat", 0.0))
                lon = float(payload.get("lon", 0.0))
                alt = float(payload.get("alt", 0.0))
                with open("/dev/shm/icarmeter_phone_telemetry", "w", encoding="utf-8") as f:
                    f.write(f"spd={spd}\nlat={lat}\nlon={lon}\nalt={alt}\n")
                touch_app_connected("Wi-Fi Mobile App")
                log_ws_event("Telemetry")
                self.send_json(200, {"status": "ok"})
            except Exception as e:
                self.send_json(400, {"status": "error", "message": str(e)})
            return

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
    # Start WebSocket background heartbeat worker
    t_hb = threading.Thread(target=ws_heartbeat_worker, daemon=True)
    t_hb.start()

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
