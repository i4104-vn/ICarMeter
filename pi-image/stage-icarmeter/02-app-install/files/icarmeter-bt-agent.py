#!/usr/bin/env python3
"""
ICarMeter Bluetooth Auto-Accept Pairing Agent
Registers as default BlueZ agent (NoInputNoOutput) to automatically accept
incoming pairing and connection requests from the companion phone.
If a PIN or Passkey is required/generated, it is recorded to /dev/shm/icarmeter_bt_pin
so the companion app can display it to the user.
"""
import datetime
import json
import os
import re
import sys
import time
import subprocess

AGENT_INTERFACE = "org.bluez.Agent1"
AGENT_PATH = "/org/icarmeter/btagent"
PIN_MARKER = "/dev/shm/icarmeter_bt_pin"
BT_LOG_FILE = "/var/log/icarmeter-bluetooth.log"
BT_LOG_FALLBACK = "/dev/shm/icarmeter-bluetooth.log"

def log_bt(message):
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    line = f"[{timestamp}] [BTAgent] {message}"
    print(line)
    for path in (BT_LOG_FILE, BT_LOG_FALLBACK):
        try:
            with open(path, "a", encoding="utf-8") as f:
                f.write(line + "\n")
            break
        except Exception:
            continue

def record_pin(pin_code, device=""):
    try:
        # Extract clean MAC address if device is a D-Bus object path (/org/bluez/hci0/dev_AA_BB_...)
        mac = str(device)
        if "dev_" in mac:
            mac_part = mac.split("dev_")[-1]
            mac = mac_part.replace("_", ":").upper()

        data = {
            "pin": str(pin_code).strip(),
            "mac": mac,
            "timestamp": time.time()
        }
        with open(PIN_MARKER, "w", encoding="utf-8") as f:
            f.write(json.dumps(data) + "\n")
        log_bt(f"Recorded PIN/Passkey: {pin_code} for device {mac}")
    except Exception as e:
        log_bt(f"Error writing PIN marker: {e}")

def clear_pin():
    try:
        if os.path.exists(PIN_MARKER):
            os.remove(PIN_MARKER)
    except Exception:
        pass

def run_dbus_agent():
    try:
        import dbus
        import dbus.service
        import dbus.mainloop.glib
        from gi.repository import GLib

        dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
        bus = dbus.SystemBus()

        class AutoAgent(dbus.service.Object):
            @dbus.service.method(AGENT_INTERFACE, in_signature="", out_signature="")
            def Release(self):
                log_bt("Release agent callback triggered")
                clear_pin()

            @dbus.service.method(AGENT_INTERFACE, in_signature="os", out_signature="")
            def AuthorizeService(self, device, uuid):
                log_bt(f"AuthorizeService: {device} uuid: {uuid} -> Auto-Authorized")
                return

            @dbus.service.method(AGENT_INTERFACE, in_signature="o", out_signature="s")
            def RequestPinCode(self, device):
                pin = "1234"
                log_bt(f"RequestPinCode: {device} -> providing PIN {pin}")
                record_pin(pin, device)
                return pin

            @dbus.service.method(AGENT_INTERFACE, in_signature="o", out_signature="u")
            def RequestPasskey(self, device):
                passkey = 123456
                log_bt(f"RequestPasskey: {device} -> providing passkey {passkey:06d}")
                record_pin(f"{passkey:06d}", device)
                return dbus.UInt32(passkey)

            @dbus.service.method(AGENT_INTERFACE, in_signature="ou", out_signature="")
            def DisplayPasskey(self, device, passkey):
                code_str = f"{passkey:06d}"
                log_bt(f"DisplayPasskey: {device} passkey: {code_str}")
                record_pin(code_str, device)

            @dbus.service.method(AGENT_INTERFACE, in_signature="os", out_signature="")
            def DisplayPinCode(self, device, pincode):
                log_bt(f"DisplayPinCode: {device} pin: {pincode}")
                record_pin(str(pincode), device)

            @dbus.service.method(AGENT_INTERFACE, in_signature="ouq", out_signature="")
            def DisplayPasskeyAndEntered(self, device, passkey, entered):
                pass

            @dbus.service.method(AGENT_INTERFACE, in_signature="ou", out_signature="")
            def RequestConfirmation(self, device, passkey):
                code_str = f"{passkey:06d}"
                log_bt(f"RequestConfirmation: {device} passkey: {code_str} -> Auto-Accepted")
                record_pin(code_str, device)
                return

            @dbus.service.method(AGENT_INTERFACE, in_signature="o", out_signature="")
            def RequestAuthorization(self, device):
                log_bt(f"RequestAuthorization: {device} -> Auto-Accepted")
                return

            @dbus.service.method(AGENT_INTERFACE, in_signature="", out_signature="")
            def Cancel(self):
                log_bt("Cancel agent callback triggered")
                clear_pin()

        agent = AutoAgent(bus, AGENT_PATH)
        manager = dbus.Interface(
            bus.get_object("org.bluez", "/org/bluez"),
            "org.bluez.AgentManager1"
        )
        manager.RegisterAgent(AGENT_PATH, "NoInputNoOutput")
        manager.RequestDefaultAgent(AGENT_PATH)
        log_bt("Registered NoInputNoOutput agent as default agent via D-Bus")

        loop = GLib.MainLoop()
        loop.run()
    except ImportError:
        log_bt("dbus/gi not available, falling back to bluetoothctl agent loop...")
        run_bluetoothctl_agent()
    except Exception as e:
        log_bt(f"D-Bus registration error ({e}), falling back to bluetoothctl...")
        run_bluetoothctl_agent()

def run_bluetoothctl_agent():
    """Fallback agent using bluetoothctl daemon process"""
    while True:
        try:
            p = subprocess.Popen(
                ["bluetoothctl"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True
            )
            commands = "agent NoInputNoOutput\ndefault-agent\npower on\npairable on\n"
            p.stdin.write(commands)
            p.stdin.flush()
            log_bt("Fallback bluetoothctl agent active")
            while True:
                line = p.stdout.readline()
                if not line:
                    break
                line_str = line.strip()
                # Check for passkey or PIN in output
                m = re.search(r"(?:Confirm passkey|Passkey|PIN code)\s*[:=]?\s*(\d{4,6})", line_str, re.IGNORECASE)
                if m:
                    code = m.group(1)
                    record_pin(code)

                if "Authorize service" in line_str or "Confirm passkey" in line_str or "Request confirmation" in line_str:
                    p.stdin.write("yes\n")
                    p.stdin.flush()
                elif "Enter PIN code" in line_str:
                    p.stdin.write("1234\n")
                    p.stdin.flush()
                    record_pin("1234")
            p.wait()
        except Exception as e:
            log_bt(f"bluetoothctl agent loop error: {e}")
        time.sleep(3)

if __name__ == "__main__":
    run_dbus_agent()
