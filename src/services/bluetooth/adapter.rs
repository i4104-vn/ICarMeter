//! Host Bluetooth controller query and D-Bus management utilities for BlueZ.

use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue};

pub const DEFAULT_ADAPTER: &str = "hci0";
const BLUEZ_SERVICE: &str = "org.bluez";
const ADAPTER_INTERFACE: &str = "org.bluez.Adapter1";
const DEVICE_INTERFACE: &str = "org.bluez.Device1";
const OBJECT_MANAGER_INTERFACE: &str = "org.freedesktop.DBus.ObjectManager";

/// Extracts the local adapter's Bluetooth MAC address directly via D-Bus or /sys filesystem.
pub fn get_local_mac(adapter: &str) -> String {
    // 1. Try D-Bus query to org.bluez.Adapter1.Address
    if let Ok(conn) = Connection::system() {
        let path = format!("/org/bluez/{}", adapter);
        if let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, path.as_str(), ADAPTER_INTERFACE) {
            if let Ok(addr) = proxy.get_property::<String>("Address") {
                let clean = addr.trim().to_uppercase();
                if !clean.is_empty() {
                    return clean;
                }
            }
        }
    }

    // 2. Direct sysfs fallback
    let sys_path = format!("/sys/class/bluetooth/{}/address", adapter);
    if let Ok(content) = fs::read_to_string(&sys_path) {
        let mac = content.trim().to_uppercase();
        if !mac.is_empty() {
            return mac;
        }
    }

    "00:00:00:00:00:00".to_string()
}

/// Ensures Bluetooth adapter is powered, named ICarMeter, discoverable, and pairable via BlueZ D-Bus.
pub fn ensure_adapter_ready(adapter: &str) {
    if let Ok(conn) = Connection::system() {
        let path = format!("/org/bluez/{}", adapter);
        if let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, path.as_str(), ADAPTER_INTERFACE) {
            let _ = proxy.set_property("Powered", true);
            let _ = proxy.set_property("Alias", "ICarMeter");
            let _ = proxy.set_property("DiscoverableTimeout", 0u32);
            let _ = proxy.set_property("PairableTimeout", 0u32);
            let _ = proxy.set_property("Discoverable", true);
            let _ = proxy.set_property("Pairable", true);
        }
    }
}

/// Queries BlueZ ObjectManager over D-Bus to retrieve all paired devices.
pub fn get_paired_devices() -> Vec<Value> {
    let mut devices = Vec::new();
    let Ok(conn) = Connection::system() else {
        return devices;
    };

    let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, "/", OBJECT_MANAGER_INTERFACE) else {
        return devices;
    };

    type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;
    let Ok(reply) = proxy.call_method("GetManagedObjects", &()) else {
        return devices;
    };
    let Ok(objects) = reply.body().deserialize::<ManagedObjects>() else {
        return devices;
    };

    for (_path, interfaces) in objects {
        if let Some(dev_props) = interfaces.get(DEVICE_INTERFACE) {
            let is_paired = dev_props
                .get("Paired")
                .map_or(false, |v| matches!(&**v, zbus::zvariant::Value::Bool(true)));

            if is_paired {
                let mac = dev_props
                    .get("Address")
                    .and_then(|v| match &**v {
                        zbus::zvariant::Value::Str(s) => Some(s.as_str().to_uppercase()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let name = dev_props
                    .get("Alias")
                    .or_else(|| dev_props.get("Name"))
                    .and_then(|v| match &**v {
                        zbus::zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| format!("Device ({})", mac));

                devices.push(json!({
                    "mac": mac,
                    "name": name,
                    "paired": true
                }));
            }
        }
    }

    devices
}

/// Starts Bluetooth discovery scan using BlueZ D-Bus Adapter1.StartDiscovery.
pub fn start_discovery() {
    if let Ok(conn) = Connection::system() {
        if let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, "/org/bluez/hci0", ADAPTER_INTERFACE) {
            let _ = proxy.call_method("StartDiscovery", &());
        }
    }
}

/// Stops Bluetooth discovery scan using BlueZ D-Bus Adapter1.StopDiscovery.
pub fn stop_discovery() {
    if let Ok(conn) = Connection::system() {
        if let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, "/org/bluez/hci0", ADAPTER_INTERFACE) {
            let _ = proxy.call_method("StopDiscovery", &());
        }
    }
}

/// Removes a paired Bluetooth device using BlueZ D-Bus Adapter1.RemoveDevice.
pub fn unpair_device(mac: &str) -> bool {
    let Ok(conn) = Connection::system() else {
        return false;
    };
    let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, "/org/bluez/hci0", ADAPTER_INTERFACE) else {
        return false;
    };

    let dev_subpath = mac.replace(':', "_").to_uppercase();
    let dev_obj_path = format!("/org/bluez/hci0/dev_{}", dev_subpath);
    let Ok(obj_path) = ObjectPath::try_from(dev_obj_path.as_str()) else {
        return false;
    };

    proxy.call_method("RemoveDevice", &(obj_path,)).is_ok()
}

/// Pairs a target Bluetooth device using BlueZ D-Bus Device1.Pair.
pub fn pair_device(mac: &str) -> (bool, String) {
    let Ok(conn) = Connection::system() else {
        return (false, "D-Bus system bus unavailable".to_string());
    };

    let dev_subpath = mac.replace(':', "_").to_uppercase();
    let dev_obj_path = format!("/org/bluez/hci0/dev_{}", dev_subpath);

    let Ok(proxy) = Proxy::new(
        &conn,
        BLUEZ_SERVICE,
        dev_obj_path.as_str(),
        DEVICE_INTERFACE,
    ) else {
        return (false, "Device not found on D-Bus".to_string());
    };

    match proxy.call_method("Pair", &()) {
        Ok(_) => (true, "Pairing successful".to_string()),
        Err(e) => (false, e.to_string()),
    }
}
