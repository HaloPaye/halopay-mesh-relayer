use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::gossip::AckPayload;
use ed25519_dalek::SigningKey;
use mesh_crypto::encrypt_payload;
use mesh_protocol::{build_packets, MsgType};
use mesh_storage::Storage;
use mesh_transport::Transport;

#[derive(serde::Serialize)]
struct SettleRequest {
    payloads: Vec<String>, // base64
}

#[derive(serde::Deserialize)]
struct SettleResponse {
    settled: Vec<String>,
    rejected: Vec<String>,
}

pub struct SettlementClient {
    storage: Arc<Mutex<Storage>>,
    transport: Arc<dyn Transport>,
    keypair: SigningKey,
    api_url: String,
    client: Client,
    pub tui_tx: Option<tokio::sync::mpsc::Sender<String>>,
}

impl SettlementClient {
    pub fn new(
        storage: Arc<Mutex<Storage>>,
        transport: Arc<dyn Transport>,
        keypair: SigningKey,
    ) -> Self {
        let api_url = env::var("API_URL").unwrap_or_else(|_| "https://api.halopay.app".to_string());
        if api_url.is_empty() {
            panic!("API_URL environment variable is required");
        }

        Self {
            storage,
            transport,
            keypair,
            api_url,
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            tui_tx: None,
        }
    }

    pub async fn run(&self) {
        let ping_url = format!("{}/ping", self.api_url);
        let settle_url = format!("{}/settle", self.api_url);

        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            let res = self.client.get(&ping_url).send().await;
            match res {
                Ok(resp) if resp.status().is_success() => {
                    self.process_settlement(&settle_url).await;
                }
                _ => {}
            }
        }
    }

    pub async fn process_settlement(&self, settle_url: &str) {
        let pending = {
            let storage = self.storage.lock().await;
            storage.get_pending_txs().unwrap_or_default()
        };

        if pending.is_empty() {
            return;
        }

        let mut payloads_b64 = Vec::new();
        let mut hashes = Vec::new();
        for tx in &pending {
            payloads_b64.push(general_purpose::STANDARD.encode(&tx.payload));
            hashes.push(tx.hash.clone());
        }

        let req = SettleRequest {
            payloads: payloads_b64,
        };

        let res = self.client.post(settle_url).json(&req).send().await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(parsed) = resp.json::<SettleResponse>().await {
                    let mut storage = self.storage.lock().await;

                    for hash in parsed.settled {
                        let _ = storage.update_status(&hash, "settled");
                        self.log_and_gossip_ack(&hash, "settled").await;
                    }

                    for hash in parsed.rejected {
                        let _ = storage.update_status(&hash, "failed");
                        self.log_and_gossip_ack(&hash, "failed").await;
                    }
                } else {
                    let msg = r#"{{"event": "API_ERROR", "action": "RETRY_IN_60S"}}"#.to_string();
                    if let Some(tx) = &self.tui_tx {
                        let _ = tx.send(msg).await;
                    } else {
                        println!("{}", msg);
                    }
                }
            }
            _ => {
                let msg = r#"{{"event": "API_ERROR", "action": "RETRY_IN_60S"}}"#.to_string();
                if let Some(tx) = &self.tui_tx {
                    let _ = tx.send(msg).await;
                } else {
                    println!("{}", msg);
                }
            }
        }
    }

    async fn log_and_gossip_ack(&self, hash: &str, status: &str) {
        let node_id = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            self.keypair.verifying_key().to_bytes()[0],
            self.keypair.verifying_key().to_bytes()[1],
            self.keypair.verifying_key().to_bytes()[2],
            self.keypair.verifying_key().to_bytes()[3]
        );
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let msg = if status == "settled" {
            format!(
                r#"{{"timestamp": {}, "node_id": "{}", "event": "API_SUBMIT_SUCCESS", "tx_hash": "{}"}}"#,
                now, node_id, hash
            )
        } else {
            format!(
                r#"{{"timestamp": {}, "node_id": "{}", "event": "API_SUBMIT_FAILED", "tx_hash": "{}"}}"#,
                now, node_id, hash
            )
        };

        if let Some(tx) = &self.tui_tx {
            let _ = tx.send(msg).await;
        } else {
            println!("{}", msg);
        }

        let ack = AckPayload {
            hash: hash.to_string(),
            status: status.to_string(),
        };
        let ack_json = serde_json::to_vec(&ack).unwrap();
        let encrypted_ack = encrypt_payload(&ack_json, now).unwrap();

        let packets = build_packets(&self.keypair, MsgType::SettlementAck, &encrypted_ack);
        for p in packets {
            let _ = self.transport.broadcast(&p.encode()).await;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
}
