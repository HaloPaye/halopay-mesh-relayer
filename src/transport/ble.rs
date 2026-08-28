#[cfg(feature = "hardware")]
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
#[cfg(feature = "hardware")]
use btleplug::platform::Manager;

use super::{Transport, TransportError};
use async_trait::async_trait;

#[cfg(feature = "hardware")]
pub struct BleTransport {
    pub node_id: [u8; 32],
}

#[cfg(feature = "hardware")]
impl BleTransport {
    pub async fn new(node_id: [u8; 32]) -> Self {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        if adapters.is_empty() {
            println!(r#"{{"event": "HW_ERROR", "fallback": "sim"}}"#);
            panic!("No Bluetooth adapters found");
        }
        Self { node_id }
    }
}

#[cfg(feature = "hardware")]
#[async_trait]
impl Transport for BleTransport {
    async fn broadcast(&self, _data: &[u8]) -> Result<(), TransportError> {
        // Implementation for BLE broadcast (e.g. advertising or writing to characteristic)
        Ok(())
    }

    async fn receive(&self) -> Result<([u8; 32], Vec<u8>), TransportError> {
        // Implementation for receiving BLE notifications
        std::future::pending().await
    }
}

#[cfg(not(feature = "hardware"))]
pub struct BleTransport {}

#[cfg(not(feature = "hardware"))]
impl BleTransport {
    pub async fn new(_node_id: [u8; 32]) -> Self {
        panic!("BleTransport requires the 'hardware' feature");
    }
}

#[cfg(not(feature = "hardware"))]
#[async_trait]
impl Transport for BleTransport {
    async fn broadcast(&self, _data: &[u8]) -> Result<(), TransportError> {
        unimplemented!()
    }

    async fn receive(&self) -> Result<([u8; 32], Vec<u8>), TransportError> {
        unimplemented!()
    }
}
