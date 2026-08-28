use super::{Transport, TransportError};
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::sync::Arc;

pub struct SimTransport {
    pub node_id: [u8; 32],
    pub sender: broadcast::Sender<([u8; 32], Vec<u8>)>,
    pub receiver: tokio::sync::Mutex<broadcast::Receiver<([u8; 32], Vec<u8>)>>,
}

impl SimTransport {
    pub fn new(node_id: [u8; 32], sender: broadcast::Sender<([u8; 32], Vec<u8>)>) -> Self {
        let receiver = sender.subscribe();
        Self {
            node_id,
            sender,
            receiver: tokio::sync::Mutex::new(receiver),
        }
    }
}

#[async_trait]
impl Transport for SimTransport {
    async fn broadcast(&self, data: &[u8]) -> Result<(), TransportError> {
        let _ = self.sender.send((self.node_id, data.to_vec()));
        Ok(())
    }

    async fn receive(&self) -> Result<([u8; 32], Vec<u8>), TransportError> {
        let mut rx = self.receiver.lock().await;
        loop {
            match rx.recv().await {
                Ok((sender_id, data)) => {
                    // Ignore our own broadcasts
                    if sender_id != self.node_id {
                        return Ok((sender_id, data));
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(e) => return Err(TransportError::RecvFailed(e.to_string())),
            }
        }
    }
}
