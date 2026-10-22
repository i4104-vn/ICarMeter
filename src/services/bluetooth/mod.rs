//! Dedicated Bluetooth RFCOMM service for vehicle companion mobile application.
//!
//! Provides a native, high-performance RFCOMM SPP socket server and BlueZ D-Bus integration.

pub mod adapter;
pub mod handler;
pub mod protocol;
pub mod server;
pub mod socket;

pub use server::RfcommServer as BluetoothServer;
