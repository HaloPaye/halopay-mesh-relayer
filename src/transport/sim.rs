use super::{Transport, TransportError};
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::sync::Arc;

pub struct SimTransport {
    pub node_id: [u8; 32],
    pub sender: broadcast::Sender<([u8; 32], Vec<u8>)>,
    pub receiver: tokio::sync::Mutex<broadcast::Receiver<([u8; 32], Vec<u8>)>>,
    pub connected_peers: Arc<std::sync::RwLock<std::collections::HashSet<[u8; 32]>>>,
}

impl SimTransport {
    pub fn new(node_id: [u8; 32], sender: broadcast::Sender<([u8; 32], Vec<u8>)>) -> Self {
        let receiver = sender.subscribe();
        Self {
            node_id,
            sender,
            receiver: tokio::sync::Mutex::new(receiver),
            connected_peers: Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        }
    }
    
    pub fn connect_to(&self, peer_id: [u8; 32]) {
        self.connected_peers.write().unwrap().insert(peer_id);
    }
    
    pub fn disconnect_from(&self, peer_id: [u8; 32]) {
        self.connected_peers.write().unwrap().remove(&peer_id);
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
                    if sender_id != self.node_id {
                        let is_connected = {
                            let peers = self.connected_peers.read().unwrap();
                            peers.is_empty() || peers.contains(&sender_id) // if empty, assume fully connected for convenience, or strictly check. Let's strictly check.
                            // Actually, wait, let's make it so if we use `connected_peers`, we only receive from them.
                        };
                        // To make it fully connected by default, let's just say if connected_peers is populated we use it. But it's easier if we explicitly connect.
                        // Let's assume connected if it's in the set.
                        if self.connected_peers.read().unwrap().contains(&sender_id) {
                            return Ok((sender_id, data));
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(e) => return Err(TransportError::RecvFailed(e.to_string())),
            }
        }
    }
}
