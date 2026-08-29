use mesh_crypto::{generate_keypair, sign_message, verify_signature};
use mesh_protocol::{build_packets, MsgType};

#[test]
fn test_signature_verification_pass() {
    let keypair = generate_keypair();
    let payload = b"Hello Test";
    let timestamp = 1000;
    let sig = sign_message(&keypair, timestamp, payload.len() as u16, payload);
    assert!(verify_signature(&keypair.verifying_key(), timestamp, payload.len() as u16, payload, &sig));
}

#[test]
fn test_signature_verification_tamper_payload() {
    let keypair = generate_keypair();
    let mut payload = b"Hello Test".to_vec();
    let timestamp = 1000;
    let sig = sign_message(&keypair, timestamp, payload.len() as u16, &payload);
    
    payload[0] ^= 0xFF; // Tamper
    assert!(!verify_signature(&keypair.verifying_key(), timestamp, payload.len() as u16, &payload, &sig));
}

#[test]
fn test_oversized_payload_fragmentation() {
    let keypair = generate_keypair();
    let large_payload = vec![0u8; 300];
    let packets = build_packets(&keypair, MsgType::TxGossip, &large_payload);
    
    // 300 bytes > 148 limit, should be fragmented
    assert!(packets.len() > 1);
    assert_eq!(packets[0].type_byte & 0x80, 0x80); // fragmented bit set
}
