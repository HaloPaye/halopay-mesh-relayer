use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::protocol::{Packet, MsgType, build_packets, is_timestamp_valid};
use crate::crypto::{verify_signature, hash_payload, decrypt_payload, encrypt_payload};
use crate::storage::Storage;
use crate::transport::Transport;
use ed25519_dalek::{VerifyingKey, Signature, SigningKey};
use blake3::Hash;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TxPayload {
    pub nonce: u64,
    pub amount_usdc: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AckPayload {
    pub hash: String,
    pub status: String,
}

pub struct GossipNode {
    pub keypair: SigningKey,
    pub transport: Arc<dyn Transport>,
    pub storage: Arc<Mutex<Storage>>,
    pub lru_cache: Mutex<VecDeque<String>>,
    pub double_spend_cache: Mutex<HashMap<([u8; 32], u64), [u8; 64]>>,
    pub fragments: Mutex<HashMap<[u8; 32], HashMap<u8, Vec<u8>>>>, // Sender -> ChunkIndex -> Data
}

impl GossipNode {
    pub fn new(keypair: SigningKey, transport: Arc<dyn Transport>, storage: Arc<Mutex<Storage>>) -> Self {
        Self {
            keypair,
            transport,
            storage,
            lru_cache: Mutex::new(VecDeque::with_capacity(10000)),
            double_spend_cache: Mutex::new(HashMap::new()),
            fragments: Mutex::new(HashMap::new()),
        }
    }

    pub async fn log(&self, event: &str, tx_hash: Option<&str>) {
        let pubkey_bytes = self.keypair.verifying_key().to_bytes();
        let node_id = format!("{:02x}{:02x}{:02x}{:02x}", pubkey_bytes[0], pubkey_bytes[1], pubkey_bytes[2], pubkey_bytes[3]);
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        if let Some(h) = tx_hash {
            println!(r#"{{"timestamp": {}, "node_id": "{}", "event": "{}", "tx_hash": "{}"}}"#, now, node_id, event, h);
        } else {
            println!(r#"{{"timestamp": {}, "node_id": "{}", "event": "{}"}}"#, now, node_id, event);
        }
    }

    pub async fn run(&self) {
        // Startup Replay
        {
            let storage = self.storage.lock().await;
            if let Ok(pending) = storage.get_pending_txs() {
                for tx_record in pending {
                    if let Ok(packet) = Packet::decode(&tx_record.payload) {
                        let relay_packets = crate::protocol::fragment_packet(&packet);
                        let trans_clone = self.transport.clone();
                        tokio::spawn(async move {
                            for p in relay_packets {
                                let _ = trans_clone.broadcast(&p.encode()).await;
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            }
                        });
                    }
                }
            }
        }

        // Hello broadcast background task
        let kp_clone = self.keypair.clone();
        let trans_clone = self.transport.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
                let payload = b"Hello";
                let packets = build_packets(&kp_clone, MsgType::Hello, payload);
                for p in packets {
                    let _ = trans_clone.broadcast(&p.encode()).await;
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        });

        loop {
            match self.transport.receive().await {
                Ok((sender_id, data)) => {
                    self.handle_raw_data(sender_id, &data).await;
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }
    }

    fn check_battery() -> u8 {
        // Fallback: Read BATTERY_LEVEL env var or assume 100%
        std::env::var("BATTERY_LEVEL").unwrap_or_else(|_| "100".to_string()).parse().unwrap_or(100)
    }

    async fn handle_raw_data(&self, sender_id: [u8; 32], data: &[u8]) {
        let packet = match Packet::decode(data) {
            Ok(p) => p,
            Err(_) => return,
        };

        if packet.version != 0x01 {
            return;
        }

        if !is_timestamp_valid(packet.timestamp) {
            return;
        }

        if packet.is_fragmented() {
            let chunk_index = packet.payload[0];
            let total_chunks = packet.payload[1];
            let chunk_data = &packet.payload[2..];
            
            let mut frags = self.fragments.lock().await;
            let sender_frags = frags.entry(sender_id).or_insert_with(HashMap::new);
            sender_frags.insert(chunk_index, chunk_data.to_vec());
            
            if sender_frags.len() as u8 == total_chunks {
                let mut reassembled = Vec::new();
                for i in 0..total_chunks {
                    if let Some(c) = sender_frags.get(&i) {
                        reassembled.extend_from_slice(c);
                    }
                }
                frags.remove(&sender_id);
                
                let mut full_packet = packet.clone();
                full_packet.payload = reassembled;
                full_packet.payload_length = full_packet.payload.len() as u16;
                full_packet.type_byte &= 0x7F; // Remove fragmentation flag
                Box::pin(self.process_packet(full_packet)).await;
            }
            return;
        }

        Box::pin(self.process_packet(packet)).await;
    }

    async fn process_packet(&self, packet: Packet) {
        let vk = match VerifyingKey::from_bytes(&packet.sender_pubkey) {
            Ok(k) => k,
            Err(_) => return,
        };

        let sig = match Signature::from_slice(&packet.signature) {
            Ok(s) => s,
            Err(_) => return,
        };

        if !verify_signature(&vk, packet.timestamp, packet.payload_length, &packet.payload, &sig) {
            return;
        }

        let payload_hash = hash_payload(&packet.payload);
        let hash_hex = payload_hash.to_hex().as_str().to_string();

        {
            let mut lru = self.lru_cache.lock().await;
            if lru.contains(&hash_hex) {
                return; // Duplicate
            }
            if lru.len() >= 10000 {
                lru.pop_front();
            }
            lru.push_back(hash_hex.clone());
        }

        let msg_type = match MsgType::from_u8(packet.type_byte) {
            Some(t) => t,
            None => return,
        };

        match msg_type {
            MsgType::Hello => {
                // Ignore for now
            }
            MsgType::TxGossip => {
                self.log("GOSSIP_RECEIVED", Some(&hash_hex)).await;

                let decrypted = match decrypt_payload(&packet.payload, packet.timestamp) {
                    Ok(d) => d,
                    Err(_) => return,
                };
                
                let tx_payload: TxPayload = match serde_json::from_slice(&decrypted) {
                    Ok(p) => p,
                    Err(_) => return,
                };

                // Conflict Detection (Double Spend)
                {
                    let mut ds_cache = self.double_spend_cache.lock().await;
                    let key = (packet.sender_pubkey, tx_payload.nonce);
                    if let Some(existing_sig) = ds_cache.get(&key) {
                        let hash1 = blake3::hash(&packet.signature);
                        let hash2 = blake3::hash(existing_sig);
                        if hash1.as_bytes() > hash2.as_bytes() {
                            // We lose
                            self.log("DOUBLE_SPEND_DETECTED_LOST", Some(&hash_hex)).await;
                            
                            // Generate SettlementFailed (SettlementAck with failed status)
                            let fail_ack = AckPayload {
                                hash: hash_hex.clone(),
                                status: "failed".to_string(),
                            };
                            let ack_json = serde_json::to_vec(&fail_ack).unwrap();
                            let encrypted_ack = encrypt_payload(&ack_json, packet.timestamp).unwrap();
                            let ack_packets = build_packets(&self.keypair, MsgType::SettlementAck, &encrypted_ack);
                            for p in ack_packets {
                                let _ = self.transport.broadcast(&p.encode()).await;
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            }
                            return;
                        } else {
                            // We win, overwrite
                            ds_cache.insert(key, packet.signature);
                        }
                    } else {
                        ds_cache.insert(key, packet.signature);
                    }
                }

                // Offline Exposure Limits
                let mut sum_usdc = 0.0;
                {
                    let storage = self.storage.lock().await;
                    if let Ok(pending) = storage.get_pending_txs() {
                        for p_tx in pending {
                            if let Ok(db_packet) = Packet::decode(&p_tx.payload) {
                                if db_packet.sender_pubkey == packet.sender_pubkey {
                                    if let Ok(dec_db) = decrypt_payload(&db_packet.payload, db_packet.timestamp) {
                                        if let Ok(db_tx_payload) = serde_json::from_slice::<TxPayload>(&dec_db) {
                                            sum_usdc += db_tx_payload.amount_usdc;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                if sum_usdc + tx_payload.amount_usdc > 500.0 {
                    self.log("OFFLINE_LIMIT_EXCEEDED", Some(&hash_hex)).await;
                    return;
                }

                {
                    let mut storage = self.storage.lock().await;
                    let _ = storage.insert_pending_tx(&hash_hex, &packet.encode()); // STORE FULL ENCODED PACKET
                }

                // Broadcast to peers if battery >= 15 or if it's our own transaction
                let is_own = packet.sender_pubkey == self.keypair.verifying_key().to_bytes();
                if is_own || Self::check_battery() >= 15 {
                    let relay_packets = crate::protocol::fragment_packet(&packet);
                    for p in relay_packets {
                        let _ = self.transport.broadcast(&p.encode()).await;
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                } else {
                    self.log("BATTERY_LOW_SKIPPED_RELAY", Some(&hash_hex)).await;
                }
            }
            MsgType::SettlementAck => {
                self.log("ACK_RECEIVED", Some(&hash_hex)).await;
                
                let decrypted = match decrypt_payload(&packet.payload, packet.timestamp) {
                    Ok(d) => d,
                    Err(_) => return,
                };
                
                let ack_payload: AckPayload = match serde_json::from_slice(&decrypted) {
                    Ok(p) => p,
                    Err(_) => return,
                };

                {
                    let mut storage = self.storage.lock().await;
                    let _ = storage.update_status(&ack_payload.hash, &ack_payload.status);
                }

                // Relay
                let relay_packets = crate::protocol::fragment_packet(&packet);
                for p in relay_packets {
                    let _ = self.transport.broadcast(&p.encode()).await;
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }
    }

    pub async fn inject_transaction(&self, tx: TxPayload) -> String {
        let json = serde_json::to_vec(&tx).unwrap();
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let encrypted = encrypt_payload(&json, timestamp).unwrap();
        
        let payload_hash = hash_payload(&encrypted);
        let hash = payload_hash.to_hex().as_str().to_string();
        self.log("TX_INJECTED", Some(&hash)).await;
        
        let packets = build_packets(&self.keypair, MsgType::TxGossip, &encrypted);
        for p in packets {
            // Process it locally first
            self.handle_raw_data(self.keypair.verifying_key().to_bytes(), &p.encode()).await;
        }
        hash
    }
}

