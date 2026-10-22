//! Dedicated Bluetooth RFCOMM service for vehicle companion mobile application.
//!
//! Replaces legacy external Python daemons by providing a native, high-performance
//! RFCOMM SPP socket server, D-Bus BlueZ management, and real-time telemetry sync.

pub mod adapter;
pub mod handler;
pub mod protocol;
pub mod server;
pub mod socket;
pub mod state;

pub use server::RfcommServer as BluetoothServer;
pub use state::BluetoothSharedState;
