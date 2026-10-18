#!/usr/bin/env python3
"""
ICarMeter BLE Peripheral & GATT Service
Provides Bluetooth LE Advertisement as 'ICarMeter by I4104' and GATT Server
for Mobile Companion App synchronization with /opt/icarmeter/config.toml
"""

import sys
import os
import json
import dbus
import dbus.service
import dbus.mainloop.glib
from gi.repository import GLib

BLUEZ_SERVICE_NAME = 'org.bluez'
DBUS_OM_IFACE = 'org.freedesktop.DBus.ObjectManager'
DBUS_PROP_IFACE = 'org.freedesktop.DBus.Properties'
GATT_MGR_IFACE = 'org.bluez.GattManager1'
GATT_SERVICE_IFACE = 'org.bluez.GattService1'
GATT_CHRC_IFACE = 'org.bluez.GattCharacteristic1'
LE_ADVERTISING_MGR_IFACE = 'org.bluez.LEAdvertisingManager1'
LE_ADVERTISEMENT_IFACE = 'org.bluez.LEAdvertisement1'

BLE_SERVICE_UUID      = "1c480001-0000-1000-8000-00805f9b34fb"
CHAR_TPMS_CONFIG_UUID = "1c480002-0000-1000-8000-00805f9b34fb"
CHAR_GPS_FEED_UUID    = "1c480003-0000-1000-8000-00805f9b34fb"
CHAR_SYSTEM_CTRL_UUID = "1c480004-0000-1000-8000-00805f9b34fb"
CHAR_TRIP_STATS_UUID  = "1c480005-0000-1000-8000-00805f9b34fb"

CONFIG_PATH = "/opt/icarmeter/config.toml"


def read_config_json():
    """Reads /opt/icarmeter/config.toml and formats into JSON for mobile app"""
    data = {
        "tpms": {
            "fl": "A4:C1:38:00:00:01",
            "fr": "A4:C1:38:00:00:02",
            "rl": "A4:C1:38:00:00:03",
            "rr": "A4:C1:38:00:00:04",
            "alert": 2.2
        },
        "gps": {
            "enabled": True
        }
    }
    if os.path.exists(CONFIG_PATH):
        try:
            with open(CONFIG_PATH, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line.startswith("#") or not line:
                        continue
                    if "=" in line:
                        k, v = [x.strip() for x in line.split("=", 1)]
                        v = v.split("#")[0].strip().strip('"').strip("'")
                        if k == "fl_sensor": data["tpms"]["fl"] = v
                        elif k == "fr_sensor": data["tpms"]["fr"] = v
                        elif k == "rl_sensor": data["tpms"]["rl"] = v
                        elif k == "rr_sensor": data["tpms"]["rr"] = v
                        elif k == "low_pressure_alert": data["tpms"]["alert"] = float(v)
                        elif k == "enabled" and "gps" in data:
                            data["gps"]["enabled"] = (v.lower() == "true")
        except Exception as e:
            print(f"[BLE-Server] Error reading config: {e}", file=sys.stderr)
    return json.dumps(data)


def write_config_json(payload_bytes):
    """Updates /opt/icarmeter/config.toml with new settings from mobile app"""
    try:
        payload_str = bytes(payload_bytes).decode("utf-8")
        obj = json.loads(payload_str)
        print(f"[BLE-Server] Received config update: {obj}")

        if not os.path.exists(CONFIG_PATH):
            return

        with open(CONFIG_PATH, "r", encoding="utf-8") as f:
            lines = f.readlines()

        new_lines = []
        in_tpms = False
        in_gps = False

        for line in lines:
            stripped = line.strip()
            if stripped == "[tpms]":
                in_tpms = True
                in_gps = False
            elif stripped == "[gps]":
                in_gps = True
                in_tpms = False
            elif stripped.startswith("["):
                in_tpms = False
                in_gps = False

            if in_tpms and "=" in line:
                k = line.split("=")[0].strip()
                if k == "fl_sensor" and "fl" in obj:
                    line = f'fl_sensor = "{obj["fl"]}"\n'
                elif k == "fr_sensor" and "fr" in obj:
                    line = f'fr_sensor = "{obj["fr"]}"\n'
                elif k == "rl_sensor" and "rl" in obj:
                    line = f'rl_sensor = "{obj["rl"]}"\n'
                elif k == "rr_sensor" and "rr" in obj:
                    line = f'rr_sensor = "{obj["rr"]}"\n'
                elif k == "low_pressure_alert" and "alert" in obj:
                    line = f'low_pressure_alert = {float(obj["alert"]):.1f}\n'
            elif in_gps and "=" in line:
                k = line.split("=")[0].strip()
                if k == "enabled" and "gps_enabled" in obj:
                    val_bool = "true" if obj["gps_enabled"] else "false"
                    line = f'enabled = {val_bool}\n'

            new_lines.append(line)

        with open(CONFIG_PATH, "w", encoding="utf-8") as f:
            f.writelines(new_lines)

        print("[BLE-Server] Successfully updated config.toml from mobile app.")
    except Exception as e:
        print(f"[BLE-Server] Error saving config: {e}", file=sys.stderr)


class InvalidArgsException(dbus.exceptions.DBusException):
    _dbus_error_name = 'org.freedesktop.DBus.Error.InvalidArgs'

class NotSupportedException(dbus.exceptions.DBusException):
    _dbus_error_name = 'org.bluez.Error.NotSupported'


class Advertisement(dbus.service.Object):
    PATH_BASE = '/org/bluez/icarmeter/advertisement'

    def __init__(self, bus, index, advertising_type):
        self.path = self.PATH_BASE + str(index)
        self.bus = bus
        self.ad_type = advertising_type
        self.service_uuids = [BLE_SERVICE_UUID]
        self.local_name = 'ICarMeter by I4104'
        self.include_tx_power = True
        dbus.service.Object.__init__(self, bus, self.path)

    def get_properties(self):
        properties = dict()
        properties['Type'] = self.ad_type
        properties['ServiceUUIDs'] = dbus.Array(self.service_uuids, signature='s')
        properties['LocalName'] = dbus.String(self.local_name)
        properties['Discoverable'] = dbus.Boolean(True)
        properties['IncludeTxPower'] = dbus.Boolean(self.include_tx_power)
        return {LE_ADVERTISEMENT_IFACE: properties}

    def get_path(self):
        return dbus.ObjectPath(self.path)

    @dbus.service.method(DBUS_PROP_IFACE, in_signature='s', out_signature='a{sv}')
    def GetAll(self, interface):
        if interface != LE_ADVERTISEMENT_IFACE:
            raise InvalidArgsException()
        return self.get_properties()[LE_ADVERTISEMENT_IFACE]

    @dbus.service.method(LE_ADVERTISEMENT_IFACE, in_signature='', out_signature='')
    def Release(self):
        print(f'[BLE-Server] Advertisement {self.path} released.')


class Application(dbus.service.Object):
    def __init__(self, bus):
        self.path = '/'
        self.services = []
        dbus.service.Object.__init__(self, bus, self.path)

    def get_path(self):
        return dbus.ObjectPath(self.path)

    def add_service(self, service):
        self.services.append(service)

    @dbus.service.method(DBUS_OM_IFACE, out_signature='a{oa{sa{sv}}}')
    def GetManagedObjects(self):
        response = {}
        for service in self.services:
            response[service.get_path()] = service.get_properties()
            chrcs = service.get_characteristics()
            for chrc in chrcs:
                response[chrc.get_path()] = chrc.get_properties()
        return response


class Service(dbus.service.Object):
    PATH_BASE = '/org/bluez/icarmeter/service'

    def __init__(self, bus, index, uuid, primary):
        self.path = self.PATH_BASE + str(index)
        self.bus = bus
        self.uuid = uuid
        self.primary = primary
        self.characteristics = []
        dbus.service.Object.__init__(self, bus, self.path)

    def get_properties(self):
        return {
            GATT_SERVICE_IFACE: {
                'UUID': self.uuid,
                'Primary': self.primary,
                'Characteristics': dbus.Array(
                    self.get_characteristic_paths(),
                    signature='o'
                )
            }
        }

    def get_path(self):
        return dbus.ObjectPath(self.path)

    def add_characteristic(self, characteristic):
        self.characteristics.append(characteristic)

    def get_characteristic_paths(self):
        return [c.get_path() for c in self.characteristics]

    def get_characteristics(self):
        return self.characteristics

    @dbus.service.method(DBUS_PROP_IFACE, in_signature='s', out_signature='a{sv}')
    def GetAll(self, interface):
        if interface != GATT_SERVICE_IFACE:
            raise InvalidArgsException()
        return self.get_properties()[GATT_SERVICE_IFACE]


class Characteristic(dbus.service.Object):
    def __init__(self, bus, index, uuid, flags, service):
        self.path = service.path + '/char' + str(index)
        self.bus = bus
        self.uuid = uuid
        self.service = service
        self.flags = flags
        self.value = []
        dbus.service.Object.__init__(self, bus, self.path)

    def get_properties(self):
        return {
            GATT_CHRC_IFACE: {
                'Service': self.service.get_path(),
                'UUID': self.uuid,
                'Flags': self.flags,
                'Value': dbus.Array(self.value, signature='y')
            }
        }

    def get_path(self):
        return dbus.ObjectPath(self.path)

    @dbus.service.method(DBUS_PROP_IFACE, in_signature='s', out_signature='a{sv}')
    def GetAll(self, interface):
        if interface != GATT_CHRC_IFACE:
            raise InvalidArgsException()
        return self.get_properties()[GATT_CHRC_IFACE]


class TpmsConfigCharacteristic(Characteristic):
    def __init__(self, bus, index, service):
        Characteristic.__init__(
            self, bus, index,
            CHAR_TPMS_CONFIG_UUID,
            ['read', 'write'],
            service
        )

    def ReadValue(self, options):
        json_data = read_config_json()
        return [dbus.Byte(b) for b in json_data.encode('utf-8')]

    def WriteValue(self, value, options):
        write_config_json(value)


class GpsFeedCharacteristic(Characteristic):
    def __init__(self, bus, index, service):
        Characteristic.__init__(
            self, bus, index,
            CHAR_GPS_FEED_UUID,
            ['write', 'write-without-response'],
            service
        )

    def WriteValue(self, value, options):
        # Receives tethered GPS speed/location from mobile app
        pass


class SystemCtrlCharacteristic(Characteristic):
    def __init__(self, bus, index, service):
        Characteristic.__init__(
            self, bus, index,
            CHAR_SYSTEM_CTRL_UUID,
            ['read', 'write'],
            service
        )

    def ReadValue(self, options):
        return [dbus.Byte(b) for b in b'{"status":"ok","mode":"icarmeter"}']

    def WriteValue(self, value, options):
        pass


class TripStatsCharacteristic(Characteristic):
    def __init__(self, bus, index, service):
        Characteristic.__init__(
            self, bus, index,
            CHAR_TRIP_STATS_UUID,
            ['read', 'write'],
            service
        )

    def ReadValue(self, options):
        return [dbus.Byte(b) for b in b'[]']

    def WriteValue(self, value, options):
        pass


def find_adapter(bus):
    remote_om = dbus.Interface(bus.get_object(BLUEZ_SERVICE_NAME, '/'), DBUS_OM_IFACE)
    objects = remote_om.GetManagedObjects()
    for o, p in objects.items():
        if GATT_MGR_IFACE in p and LE_ADVERTISING_MGR_IFACE in p:
            return o
    return None


def main():
    dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
    bus = dbus.SystemBus()

    adapter_path = find_adapter(bus)
    if not adapter_path:
        print('[BLE-Server] No Bluetooth adapter with GATT & LEAdvertisingManager found. Waiting...', file=sys.stderr)
        return 1

    adapter_obj = bus.get_object(BLUEZ_SERVICE_NAME, adapter_path)
    gatt_mgr = dbus.Interface(adapter_obj, GATT_MGR_IFACE)
    adv_mgr = dbus.Interface(adapter_obj, LE_ADVERTISING_MGR_IFACE)

    # 1. Setup GATT Service
    app = Application(bus)
    service = Service(bus, 0, BLE_SERVICE_UUID, True)
    service.add_characteristic(TpmsConfigCharacteristic(bus, 0, service))
    service.add_characteristic(GpsFeedCharacteristic(bus, 1, service))
    service.add_characteristic(SystemCtrlCharacteristic(bus, 2, service))
    service.add_characteristic(TripStatsCharacteristic(bus, 3, service))
    app.add_service(service)

    # 2. Setup LE Advertisement
    ad = Advertisement(bus, 0, 'peripheral')

    mainloop = GLib.MainLoop()

    def register_app_cb():
        print('[BLE-Server] GATT application registered successfully.')

    def register_app_error_cb(error):
        print(f'[BLE-Server] Failed to register GATT application: {error}', file=sys.stderr)

    def register_ad_cb():
        print("[BLE-Server] Advertisement 'ICarMeter by I4104' registered successfully.")

    def register_ad_error_cb(error):
        print(f'[BLE-Server] Failed to register advertisement: {error}', file=sys.stderr)

    gatt_mgr.RegisterApplication(app.get_path(), {},
                                 reply_handler=register_app_cb,
                                 error_handler=register_app_error_cb)

    adv_mgr.RegisterAdvertisement(ad.get_path(), {},
                                  reply_handler=register_ad_cb,
                                  error_handler=register_ad_error_cb)

    print("[BLE-Server] ICarMeter Bluetooth LE service running...")
    mainloop.run()


if __name__ == '__main__':
    sys.exit(main())
