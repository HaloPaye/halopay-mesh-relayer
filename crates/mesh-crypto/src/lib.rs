use blake3::Hash;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use rand_core::OsRng;
use std::env;

/// Generates a new Ed25519 keypair.
pub fn generate_keypair() -> SigningKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng)
}

/// Sign the timestamp, payload_length, and payload bytes.
pub fn sign_message(
    signing_key: &SigningKey,
    timestamp: u64,
    payload_length: u16,
    payload: &[u8],
) -> Signature {
    let mut data_to_sign = Vec::new();
    data_to_sign.extend_from_slice(&timestamp.to_le_bytes());
    data_to_sign.extend_from_slice(&payload_length.to_le_bytes());
    data_to_sign.extend_from_slice(payload);
    signing_key.sign(&data_to_sign)
}

/// Verify the signature.
pub fn verify_signature(
    public_key: &VerifyingKey,
    timestamp: u64,
    payload_length: u16,
    payload: &[u8],
    signature: &Signature,
) -> bool {
    let mut data_to_verify = Vec::new();
    data_to_verify.extend_from_slice(&timestamp.to_le_bytes());
    data_to_verify.extend_from_slice(&payload_length.to_le_bytes());
    data_to_verify.extend_from_slice(payload);
    public_key.verify(&data_to_verify, signature).is_ok()
}

/// BLAKE3 Hashing
pub fn hash_payload(data: &[u8]) -> Hash {
    blake3::hash(data)
}

/// Helper for Mesh Network Key derived from MESH_NETWORK_ID env var.
/// Fallback path taken: Key is blake3(MESH_NETWORK_ID).
fn get_network_key() -> Key {
    let net_id = env::var("MESH_NETWORK_ID").unwrap_or_else(|_| "0x00000000".to_string());
    let hash = blake3::hash(net_id.as_bytes());
    Key::clone_from_slice(hash.as_bytes())
}

/// Encrypt payload using ChaCha20Poly1305.
/// Fallback path taken: Nonce is timestamp padded with 4 zeros.
pub fn encrypt_payload(payload: &[u8], timestamp: u64) -> Result<Vec<u8>, String> {
    let key = get_network_key();
    let cipher = ChaCha20Poly1305::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[0..8].copy_from_slice(&timestamp.to_le_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    cipher.encrypt(nonce, payload).map_err(|e| e.to_string())
}

/// Decrypt payload using ChaCha20Poly1305.
pub fn decrypt_payload(ciphertext: &[u8], timestamp: u64) -> Result<Vec<u8>, String> {
    let key = get_network_key();
    let cipher = ChaCha20Poly1305::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[0..8].copy_from_slice(&timestamp.to_le_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::VerifyingKey;
    
    #[test]
    fn test_signature_verify() {
        let kp = generate_keypair();
        let payload = b"test payload";
        let sig = sign_message(&kp, 12345, payload.len() as u16, payload);
        assert!(verify_signature(&kp.verifying_key(), 12345, payload.len() as u16, payload, &sig));
        
        // Tampered payload
        let tampered = b"test payloae";
        assert!(!verify_signature(&kp.verifying_key(), 12345, tampered.len() as u16, tampered, &sig));
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let payload = b"secret message";
        let timestamp = 999999;
        let encrypted = encrypt_payload(payload, timestamp).unwrap();
        let decrypted = decrypt_payload(&encrypted, timestamp).unwrap();
        assert_eq!(payload, decrypted.as_slice());
    }
}

