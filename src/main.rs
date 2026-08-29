use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;
use std::path::PathBuf;
use ed25519_dalek::{SigningKey};
use rand_core::OsRng;
use std::fs;

use mesh_storage::Storage;
use mesh_transport::sim::SimTransport;
use mesh_transport::Transport;
#[cfg(feature = "hardware")]
use mesh_transport::ble::BleTransport;
use mesh_node::gossip::GossipNode;
use mesh_node::settlement::SettlementClient;

fn get_halopay_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(home).join(".halopay");
    fs::create_dir_all(&path).unwrap();
    path
}

fn load_or_generate_key() -> SigningKey {
    let key_path = get_halopay_dir().join("node.key");
    if key_path.exists() {
        let bytes = fs::read(&key_path).unwrap();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes[0..32]);
        SigningKey::from_bytes(&key_bytes)
    } else {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        fs::write(&key_path, key.to_bytes()).unwrap();
        key
    }
}

#[tokio::main]
async fn main() {
    let api_url = env::var("API_URL").unwrap_or_else(|_| "https://api.halopay.app".to_string());
    if api_url.is_empty() {
        panic!("API_URL environment variable is required");
    }

    let transport_mode = env::var("TRANSPORT_MODE").unwrap_or_else(|_| "sim".to_string());
    let _mesh_network_id = env::var("MESH_NETWORK_ID").unwrap_or_else(|_| "0x00000000".to_string());

    let keypair = load_or_generate_key();
    let db_path = get_halopay_dir().join("mesh.db");
    
    let storage = Arc::new(Mutex::new(Storage::new(&db_path).unwrap()));
    
    let transport: Arc<dyn Transport> = if transport_mode == "ble" {
        #[cfg(feature = "hardware")]
        {
            Arc::new(BleTransport::new(keypair.verifying_key().to_bytes()).await)
        }
        #[cfg(not(feature = "hardware"))]
        {
            panic!("BleTransport requires the 'hardware' feature");
        }
    } else {
        let (tx, _) = tokio::sync::broadcast::channel(1000);
        Arc::new(SimTransport::new(keypair.verifying_key().to_bytes(), tx))
    };

    let node = Arc::new(GossipNode::new(keypair.clone(), transport.clone(), storage.clone()));
    let node_clone = node.clone();
    
    tokio::spawn(async move {
        node_clone.run().await;
    });

    let settlement = SettlementClient::new(storage.clone(), transport.clone(), keypair);
    settlement.run().await;
}
