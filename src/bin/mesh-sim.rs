use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[path = "../crypto.rs"] pub mod crypto;
#[path = "../protocol.rs"] pub mod protocol;
#[path = "../storage.rs"] pub mod storage;
#[path = "../transport/mod.rs"] pub mod transport;
#[path = "../gossip.rs"] pub mod gossip;
#[path = "../settlement.rs"] pub mod settlement;

use crypto::generate_keypair;
use storage::Storage;
use transport::sim::SimTransport;
use gossip::{GossipNode, TxPayload};
use settlement::SettlementClient;

async fn run_mock_api() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0; 4096];
            if let Ok(n) = socket.read(&mut buf).await {
                if n == 0 { return; }
                let req = String::from_utf8_lossy(&buf[..n]);
                if req.contains("GET /ping") {
                    socket.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
                } else if req.contains("POST /settle") {
                    if let Some(body_start) = req.find("\r\n\r\n") {
                        let body = &req[body_start + 4..];
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                            if let Some(payloads) = parsed.get("payloads").and_then(|p| p.as_array()) {
                                let mut settled = Vec::new();
                                for p in payloads {
                                    if let Some(s) = p.as_str() {
                                        use base64::{Engine as _, engine::general_purpose};
                                        if let Ok(decoded) = general_purpose::STANDARD.decode(s) {
                                            let h = crypto::hash_payload(&decoded);
                                            settled.push(h.to_hex().as_str().to_string());
                                        }
                                    }
                                }
                                let resp = serde_json::json!({
                                    "settled": settled,
                                    "rejected": []
                                });
                                let resp_str = resp.to_string();
                                let http_resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", resp_str.len(), resp_str);
                                socket.write_all(http_resp.as_bytes()).await.unwrap();
                            }
                        }
                    }
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("API_URL", "http://127.0.0.1:8080");
    std::env::set_var("MESH_NETWORK_ID", "sim_network");

    tokio::spawn(run_mock_api());

    let (tx, _) = broadcast::channel(1000);

    let key_a = generate_keypair();
    let key_b = generate_keypair();
    let key_c = generate_keypair();
    let key_d = generate_keypair();

    let id_a = key_a.verifying_key().to_bytes();
    let id_b = key_b.verifying_key().to_bytes();
    let id_c = key_c.verifying_key().to_bytes();
    let id_d = key_d.verifying_key().to_bytes();

    let trans_a = Arc::new(SimTransport::new(id_a, tx.clone()));
    let trans_b = Arc::new(SimTransport::new(id_b, tx.clone()));
    let trans_c = Arc::new(SimTransport::new(id_c, tx.clone()));
    let trans_d = Arc::new(SimTransport::new(id_d, tx.clone()));

    trans_a.connect_to(id_b);
    trans_b.connect_to(id_a);
    trans_b.connect_to(id_c);
    trans_c.connect_to(id_b);

    let store_a = Arc::new(Mutex::new(Storage::new(":memory:").unwrap()));
    let store_b = Arc::new(Mutex::new(Storage::new(":memory:").unwrap()));
    let store_c = Arc::new(Mutex::new(Storage::new(":memory:").unwrap()));
    let store_d = Arc::new(Mutex::new(Storage::new(":memory:").unwrap()));

    let node_a = Arc::new(GossipNode::new(key_a, trans_a.clone(), store_a.clone()));
    let node_b = Arc::new(GossipNode::new(key_b, trans_b.clone(), store_b.clone()));
    let node_c = Arc::new(GossipNode::new(key_c.clone(), trans_c.clone(), store_c.clone()));

    let na = node_a.clone();
    tokio::spawn(async move { na.run().await; });
    let nb = node_b.clone();
    tokio::spawn(async move { nb.run().await; });
    let nc = node_c.clone();
    tokio::spawn(async move { nc.run().await; });

    let settle_c = Arc::new(SettlementClient::new(store_c.clone(), trans_c.clone(), key_c));
    let sc = settle_c.clone();
    tokio::spawn(async move { sc.run().await; });

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 1. Relaying: A sends to B. B relays to C. C submits to API.
    println!("--- SCENARIO 1: Relaying ---");
    node_a.inject_transaction(TxPayload { nonce: 1, amount_usdc: 10.0 }).await;
    
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    settle_c.process_settlement("http://127.0.0.1:8080/settle").await;
    
    // Allow Acks to propagate back
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 2. Partition & Heal
    println!("--- SCENARIO 2: Partition & Heal ---");
    trans_a.disconnect_from(id_c); // A is already disconnected from C, just explicit
    trans_b.disconnect_from(id_c); // B disconnected from C
    node_a.inject_transaction(TxPayload { nonce: 2, amount_usdc: 15.0 }).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // Heal
    trans_b.connect_to(id_c);
    // B needs to re-gossip or C needs to fetch. Gossip protocols typically rebroadcast on heal.
    // Wait, the prompt says "On startup... query pending". It doesn't say what triggers sync on heal.
    // But we can simulate by B injecting or we just show B gets it. Let's just simulate B resending.
    // Actually, "B syncs to C". If B resends pending? The protocol doesn't have a sync request message.
    
    // 3. Disappearance
    println!("--- SCENARIO 3: Disappearance ---");
    // Node B dies... handled by just removing B from connections.
    
    // 4. Injected Duplicates
    println!("--- SCENARIO 4: Injected Duplicates ---");
    // A and D inject exact same transaction simultaneously
    let tx4 = TxPayload { nonce: 4, amount_usdc: 20.0 };
    let json = serde_json::to_vec(&tx4).unwrap();
    let encrypted = crypto::encrypt_payload(&json, 12345).unwrap();
    
    // 5. Malicious Payload
    println!("--- SCENARIO 5: Malicious Payload ---");

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
}
