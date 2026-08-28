use async_trait::async_trait;

#[derive(Debug)]
pub enum TransportError {
    SendFailed(String),
    RecvFailed(String),
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn broadcast(&self, data: &[u8]) -> Result<(), TransportError>;
    async fn receive(&self) -> Result<([u8; 32], Vec<u8>), TransportError>;
}

pub mod sim;
// pub mod ble;
