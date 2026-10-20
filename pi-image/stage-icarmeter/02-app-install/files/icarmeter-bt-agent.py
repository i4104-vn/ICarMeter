#!/usr/bin/env python3
"""
ICarMeter Bluetooth Auto-Accept Pairing Agent
Registers as default BlueZ agent (NoInputNoOutput) to automatically accept
incoming pairing and connection requests from the companion phone.
"""
import sys
import time
import subprocess

AGENT_INTERFACE = "org.bluez.Agent1"
AGENT_PATH = "/org/icarmeter/btagent"

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
                print("[BTAgent] Released")

            @dbus.service.method(AGENT_INTERFACE, in_signature="os", out_signature="")
            def AuthorizeService(self, device, uuid):
                print(f"[BTAgent] AuthorizeService: {device} uuid: {uuid} -> Authorized")
                return

            @dbus.service.method(AGENT_INTERFACE, in_signature="o", out_signature="s")
            def RequestPinCode(self, device):
                print(f"[BTAgent] RequestPinCode: {device} -> 1234")
                return "1234"

            @dbus.service.method(AGENT_INTERFACE, in_signature="o", out_signature="u")
            def RequestPasskey(self, device):
                print(f"[BTAgent] RequestPasskey: {device} -> 000000")
                return dbus.UInt32(0)

            @dbus.service.method(AGENT_INTERFACE, in_signature="ou", out_signature="")
            def DisplayPasskey(self, device, passkey):
                print(f"[BTAgent] DisplayPasskey: {device} passkey: {passkey}")

            @dbus.service.method(AGENT_INTERFACE, in_signature="os", out_signature="")
            def DisplayPinCode(self, device, pincode):
                print(f"[BTAgent] DisplayPinCode: {device} pin: {pincode}")

            @dbus.service.method(AGENT_INTERFACE, in_signature="ouq", out_signature="")
            def DisplayPasskeyAndEntered(self, device, passkey, entered):
                pass

            @dbus.service.method(AGENT_INTERFACE, in_signature="ou", out_signature="")
            def RequestConfirmation(self, device, passkey):
                print(f"[BTAgent] RequestConfirmation: {device} passkey: {passkey} -> Accepted")
                return

            @dbus.service.method(AGENT_INTERFACE, in_signature="o", out_signature="")
            def RequestAuthorization(self, device):
                print(f"[BTAgent] RequestAuthorization: {device} -> Accepted")
                return

            @dbus.service.method(AGENT_INTERFACE, in_signature="", out_signature="")
            def Cancel(self):
                print("[BTAgent] Cancelled")

        agent = AutoAgent(bus, AGENT_PATH)
        manager = dbus.Interface(
            bus.get_object("org.bluez", "/org/bluez"),
            "org.bluez.AgentManager1"
        )
        manager.RegisterAgent(AGENT_PATH, "NoInputNoOutput")
        manager.RequestDefaultAgent(AGENT_PATH)
        print("[BTAgent] Registered NoInputNoOutput agent as default agent via D-Bus.")

        loop = GLib.MainLoop()
        loop.run()
    except ImportError:
        print("[BTAgent] dbus/gi not available, falling back to bluetoothctl agent loop...", file=sys.stderr)
        run_bluetoothctl_agent()
    except Exception as e:
        print(f"[BTAgent] D-Bus registration error ({e}), falling back to bluetoothctl...", file=sys.stderr)
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
            # Register NoInputNoOutput agent and make it default
            commands = "agent NoInputNoOutput\ndefault-agent\npower on\npairable on\n"
            p.stdin.write(commands)
            p.stdin.flush()
            print("[BTAgent] Fallback bluetoothctl agent active.")
            while True:
                line = p.stdout.readline()
                if not line:
                    break
                line_str = line.strip()
                if "Authorize service" in line_str or "Confirm passkey" in line_str or "Request confirmation" in line_str:
                    p.stdin.write("yes\n")
                    p.stdin.flush()
            p.wait()
        except Exception as e:
            print(f"[BTAgent] bluetoothctl agent loop error: {e}", file=sys.stderr)
        time.sleep(3)

if __name__ == "__main__":
    run_dbus_agent()
