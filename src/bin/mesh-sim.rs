use mesh_crypto::generate_keypair;
use mesh_node::gossip::{GossipNode, TxPayload};
use mesh_node::settlement::SettlementClient;
use mesh_storage::Storage;
use mesh_transport::sim::SimTransport;
use mesh_transport::Transport;
use mesh_tui::run_tui;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::{broadcast, Mutex};

async fn run_mock_api() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0; 4096];
            if let Ok(n) = socket.read(&mut buf).await {
                if n == 0 {
                    return;
                }
                let req = String::from_utf8_lossy(&buf[..n]);
                if req.contains("GET /ping") {
                    socket.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
                } else if req.contains("POST /settle") {
                    if let Some(body_start) = req.find("\r\n\r\n") {
                        let body = &req[body_start + 4..];
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                            if let Some(payloads) =
                                parsed.get("payloads").and_then(|p| p.as_array())
                            {
                                let mut settled = Vec::new();
                                for p in payloads {
                                    if let Some(s) = p.as_str() {
                                        use base64::{engine::general_purpose, Engine as _};
                                        if let Ok(decoded) =
                                            general_purpose::STANDARD.decode::<&str>(s)
                                        {
                                            let h = mesh_crypto::hash_payload(&decoded);
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

    let (tui_tx, tui_rx) = mpsc::channel(100);

    let mut node_a = GossipNode::new(key_a.clone(), trans_a.clone(), store_a.clone());
    node_a.tui_tx = Some(tui_tx.clone());
    let node_a = Arc::new(node_a);
    let mut node_b = GossipNode::new(key_b, trans_b.clone(), store_b.clone());
    node_b.tui_tx = Some(tui_tx.clone());
    let node_b = Arc::new(node_b);
    let mut node_c = GossipNode::new(key_c.clone(), trans_c.clone(), store_c.clone());
    node_c.tui_tx = Some(tui_tx.clone());
    let node_c = Arc::new(node_c);
    let mut node_d = GossipNode::new(key_d.clone(), trans_d.clone(), store_d.clone());
    node_d.tui_tx = Some(tui_tx.clone());
    let node_d = Arc::new(node_d);

    let na = node_a.clone();
    tokio::spawn(async move {
        na.run().await;
    });
    let nb = node_b.clone();
    tokio::spawn(async move {
        nb.run().await;
    });
    let nc = node_c.clone();
    tokio::spawn(async move {
        nc.run().await;
    });
    let nd = node_d.clone();
    tokio::spawn(async move {
        nd.run().await;
    });

    let mut settle_c = SettlementClient::new(store_c.clone(), trans_c.clone(), key_c);
    settle_c.tui_tx = Some(tui_tx.clone());
    let sc = Arc::new(settle_c);
    let sc_clone = sc.clone();
    tokio::spawn(async move {
        sc_clone.run().await;
    });

    tokio::spawn(async move {
        let _ = run_tui(tui_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 1. Relaying: A sends to B. B relays to C. C submits to API.
    println!("--- SCENARIO 1: Relaying ---");
    node_a
        .inject_transaction(TxPayload {
            nonce: 1,
            amount_usdc: 10.0,
        })
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    sc.process_settlement("http://127.0.0.1:8080/settle").await;

    // Allow Acks to propagate back
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 2. Partition & Heal
    println!("--- SCENARIO 2: Partition & Heal ---");
    trans_a.disconnect_from(id_c);
    trans_b.disconnect_from(id_c);
    trans_c.disconnect_from(id_b);
    node_a
        .inject_transaction(TxPayload {
            nonce: 2,
            amount_usdc: 15.0,
        })
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Heal
    trans_b.connect_to(id_c);
    trans_c.connect_to(id_b);
    // B syncs to C: simulate by querying DB and rebroadcasting pending.
    {
        let store = store_b.lock().await;
        if let Ok(pending) = store.get_pending_txs() {
            for p in pending {
                let _ = trans_b.broadcast(&p.payload).await; // rebroadcast full packet
            }
        }
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    sc.process_settlement("http://127.0.0.1:8080/settle").await;

    // 3. Disappearance
    println!("--- SCENARIO 3: Disappearance ---");
    trans_a.disconnect_from(id_b); // B dies
    trans_c.disconnect_from(id_b); // B dies
    node_a
        .inject_transaction(TxPayload {
            nonce: 3,
            amount_usdc: 20.0,
        })
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 4. Injected Duplicates
    let tx4 = TxPayload {
        nonce: 4,
        amount_usdc: 20.0,
    };
    let json = serde_json::to_vec(&tx4).unwrap();
    let encrypted = mesh_crypto::encrypt_payload(&json, 12345).unwrap();
    let packets_a =
        mesh_protocol::build_packets(&key_a, mesh_protocol::MsgType::TxGossip, &encrypted);

    // D injects exact same payload at same time
    let packets_d =
        mesh_protocol::build_packets(&key_a, mesh_protocol::MsgType::TxGossip, &encrypted);
    for p in &packets_a {
        let _ = trans_a.broadcast(&p.encode()).await;
    }
    for p in &packets_d {
        let _ = trans_d.broadcast(&p.encode()).await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 5. Malicious Payload
    let mut bad_packet = packets_a[0].clone();
    bad_packet.signature[0] ^= 0xFF; // flip bits
    let _ = trans_a.broadcast(&bad_packet.encode()).await;

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
}
