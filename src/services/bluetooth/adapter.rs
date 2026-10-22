//! Host Bluetooth controller query and D-Bus management utilities for BlueZ.

use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::time::Duration;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue};
use crate::services::telemetry::bt_log::log_bt;

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

/// Finds the BlueZ D-Bus object path for the adapter, querying ObjectManager if needed.
fn find_adapter_path(conn: &Connection, adapter: &str) -> String {
    let candidate = format!("/org/bluez/{}", adapter);
    let candidate_ready = if let Ok(proxy) = Proxy::new(conn, BLUEZ_SERVICE, candidate.as_str(), ADAPTER_INTERFACE) {
        proxy.get_property::<bool>("Powered").is_ok()
    } else {
        false
    };

    if candidate_ready {
        return candidate;
    }

    // Search via ObjectManager for any Adapter1 interface
    if let Ok(om_proxy) = Proxy::new(conn, BLUEZ_SERVICE, "/", OBJECT_MANAGER_INTERFACE) {
        type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;
        if let Ok(reply) = om_proxy.call_method("GetManagedObjects", &()) {
            if let Ok(objects) = reply.body().deserialize::<ManagedObjects>() {
                for (path, interfaces) in objects {
                    if interfaces.contains_key(ADAPTER_INTERFACE) {
                        return path.as_str().to_string();
                    }
                }
            }
        }
    }

    candidate
}

/// Ensures Bluetooth adapter is powered on, named 'ICarMeter', discoverable, and pairable
/// entirely via native BlueZ D-Bus calls without spawning external subprocesses.
pub fn ensure_adapter_ready(adapter: &str) {
    let conn = match Connection::system() {
        Ok(c) => c,
        Err(e) => {
            log_bt("BT-ADAPTER", &format!("Cannot connect to system D-Bus: {e}"));
            return;
        }
    };

    let path = find_adapter_path(&conn, adapter);

    // Wait and retry for Adapter1 to be available and responsive
    let mut proxy_opt = None;
    for _ in 0..15 {
        if let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, path.as_str(), ADAPTER_INTERFACE)
            && proxy.get_property::<bool>("Powered").is_ok()
        {
            proxy_opt = Some(proxy);
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }

    let proxy = match proxy_opt {
        Some(p) => p,
        None => {
            log_bt("BT-ADAPTER", &format!("Adapter {path} not ready on D-Bus"));
            return;
        }
    };

    // 1. Ensure Powered is true (wait if transitioning)
    let is_powered = proxy.get_property::<bool>("Powered").unwrap_or(false);
    if !is_powered {
        if let Err(e) = proxy.set_property("Powered", true) {
            log_bt("BT-ADAPTER", &format!("Failed to power on adapter: {e}"));
        }
        for _ in 0..15 {
            thread::sleep(Duration::from_millis(150));
            if let Ok(true) = proxy.get_property::<bool>("Powered") {
                break;
            }
        }
    }

    // 2. Configure system Alias / Name for discovery
    let _ = proxy.set_property("Alias", "ICarMeter");

    // 3. Set timeouts to 0 (indefinite discovery and pairing)
    let _ = proxy.set_property("DiscoverableTimeout", 0u32);
    let _ = proxy.set_property("PairableTimeout", 0u32);

    // 4. Enable Pairable
    if let Err(e) = proxy.set_property("Pairable", true) {
        log_bt("BT-ADAPTER", &format!("Pairable error: {e}"));
    }

    // 5. Enable Discoverable (tells BlueZ to activate Inquiry Scan in kernel mgmt)
    let mut discov_ok = false;
    for _ in 0..5 {
        match proxy.set_property("Discoverable", true) {
            Ok(_) => {
                discov_ok = true;
                break;
            }
            Err(e) => {
                log_bt("BT-ADAPTER", &format!("Discoverable retry: {e}"));
                thread::sleep(Duration::from_millis(200));
            }
        }
    }

    // 6. Verify and log active state
    let final_pwr = proxy.get_property::<bool>("Powered").unwrap_or(false);
    let final_discov = proxy.get_property::<bool>("Discoverable").unwrap_or(discov_ok);
    let final_alias = proxy.get_property::<String>("Alias").unwrap_or_else(|_| "ICarMeter".into());
    let mac = proxy.get_property::<String>("Address").unwrap_or_else(|_| get_local_mac(adapter));

    log_bt(
        "BT-ADAPTER",
        &format!("{path} ({mac}): Name='{final_alias}', Powered={final_pwr}, Discoverable={final_discov}"),
    );
}

/// Periodically verifies adapter discoverability over D-Bus and re-asserts if expired.
pub fn maintain_discoverability(adapter: &str) {
    let Ok(conn) = Connection::system() else { return; };
    let path = format!("/org/bluez/{}", adapter);
    let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, path.as_str(), ADAPTER_INTERFACE) else { return; };

    let is_discov = proxy.get_property::<bool>("Discoverable").unwrap_or(false);
    let is_powered = proxy.get_property::<bool>("Powered").unwrap_or(false);

    if !is_powered {
        let _ = proxy.set_property("Powered", true);
        thread::sleep(Duration::from_millis(200));
    }

    if !is_discov {
        let _ = proxy.set_property("DiscoverableTimeout", 0u32);
        let _ = proxy.set_property("PairableTimeout", 0u32);
        let _ = proxy.set_property("Pairable", true);
        if proxy.set_property("Discoverable", true).is_ok() {
            log_bt("BT-ADAPTER", &format!("{adapter} Discoverable re-asserted via D-Bus"));
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
                .is_some_and(|v| matches!(&**v, zbus::zvariant::Value::Bool(true)));

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
    if let Ok(conn) = Connection::system()
        && let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, "/org/bluez/hci0", ADAPTER_INTERFACE)
    {
        let _ = proxy.call_method("StartDiscovery", &());
    }
}

/// Stops Bluetooth discovery scan using BlueZ D-Bus Adapter1.StopDiscovery.
pub fn stop_discovery() {
    if let Ok(conn) = Connection::system()
        && let Ok(proxy) = Proxy::new(&conn, BLUEZ_SERVICE, "/org/bluez/hci0", ADAPTER_INTERFACE)
    {
        let _ = proxy.call_method("StopDiscovery", &());
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
