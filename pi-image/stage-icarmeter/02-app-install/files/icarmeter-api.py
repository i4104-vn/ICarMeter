#!/usr/bin/env python3
"""
ICarMeter Headless Wi-Fi Configuration API Service
Listens for JSON API requests from the ICarMeter Mobile App over Wi-Fi Hotspot (192.168.4.1:8080)
to read, write, and synchronize /opt/icarmeter/config.toml in real time.
"""

import http.server
import json
import os
import sys
import time
import urllib.parse

PORT = 8080
CONFIG_PATH = "/opt/icarmeter/config.toml"
FALLBACK_CONFIG_PATHS = ["/home/pi/config.toml", "./config.toml"]
APP_CONN_MARKER = "/dev/shm/icarmeter_app_connected"

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
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_cors_headers()
        self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path == "/api/config":
            touch_app_connected("Wi-Fi Mobile App")
            data = read_config()
            self.send_json(200, data)
            return

        if path == "/api/status":
            touch_app_connected("Wi-Fi Mobile App")
            self.send_json(200, {
                "status": "online",
                "app_connected": True,
                "timestamp": int(time.time()),
            })
            return

        self.send_error(404, "Not Found")

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        content_length = int(self.headers.get("Content-Length", 0))
        post_body = self.rfile.read(content_length)

        if path == "/api/config":
            try:
                payload = json.loads(post_body.decode("utf-8"))
                write_config(payload)
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


def main():
    server_address = ("", PORT)
    httpd = http.server.ThreadingHTTPServer(server_address, ApiHandler)
    print(f"[API] ICarMeter headless API server running on port {PORT}...")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()


if __name__ == "__main__":
    main()
