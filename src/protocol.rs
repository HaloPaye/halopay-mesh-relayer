use ed25519_dalek::{VerifyingKey, Signature, SigningKey};
use std::convert::TryInto;
use crate::crypto::{sign_message, verify_signature};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum MsgType {
    Hello = 0x00,
    TxGossip = 0x01,
    SettlementAck = 0x02,
}

impl MsgType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v & 0x7F {
            0x00 => Some(MsgType::Hello),
            0x01 => Some(MsgType::TxGossip),
            0x02 => Some(MsgType::SettlementAck),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub version: u8,
    pub type_byte: u8,
    pub sender_pubkey: [u8; 32],
    pub timestamp: u64,
    pub signature: [u8; 64],
    pub payload_length: u16,
    pub payload: Vec<u8>, // Contains chunk index/total if fragmented
}

impl Packet {
    pub fn is_fragmented(&self) -> bool {
        (self.type_byte & 0x80) != 0
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.version);
        buf.push(self.type_byte);
        buf.extend_from_slice(&self.sender_pubkey);
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.signature);
        buf.extend_from_slice(&self.payload_length.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, String> {
        if data.len() < 108 {
            return Err("Packet too short".to_string());
        }
        let version = data[0];
        if version != 0x01 {
            return Err("Unsupported version".to_string());
        }
        let type_byte = data[1];
        let mut sender_pubkey = [0u8; 32];
        sender_pubkey.copy_from_slice(&data[2..34]);
        
        let timestamp = u64::from_le_bytes(data[34..42].try_into().unwrap());
        
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&data[42..106]);
        
        let payload_length = u16::from_le_bytes(data[106..108].try_into().unwrap());
        
        let payload = if data.len() > 108 {
            data[108..].to_vec()
        } else {
            Vec::new()
        };
        
        if payload.len() != payload_length as usize {
            return Err("Payload length mismatch".to_string());
        }
        
        Ok(Packet {
            version,
            type_byte,
            sender_pubkey,
            timestamp,
            signature,
            payload_length,
            payload,
        })
    }
}

/// Helper to build packets from a logical message
pub fn build_packets(
    signing_key: &SigningKey,
    msg_type: MsgType,
    logical_payload: &[u8], // Already encrypted
) -> Vec<Packet> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    let signature = sign_message(
        signing_key,
        timestamp,
        logical_payload.len() as u16,
        logical_payload
    );
    
    let max_packet_size = 256;
    let header_size = 108;
    let max_payload_per_packet = max_packet_size - header_size;
    
    if header_size + logical_payload.len() <= max_packet_size {
        // Single packet
        let p = Packet {
            version: 0x01,
            type_byte: msg_type as u8,
            sender_pubkey: signing_key.verifying_key().to_bytes(),
            timestamp,
            signature: signature.to_bytes(),
            payload_length: logical_payload.len() as u16,
            payload: logical_payload.to_vec(),
        };
        vec![p]
    } else {
        // Fragmented
        let chunk_header_size = 2; // index, total
        let max_chunk_data = max_payload_per_packet - chunk_header_size;
        let total_chunks = (logical_payload.len() + max_chunk_data - 1) / max_chunk_data;
        
        let mut packets = Vec::new();
        for i in 0..total_chunks {
            let start = i * max_chunk_data;
            let end = std::cmp::min(start + max_chunk_data, logical_payload.len());
            let chunk_data = &logical_payload[start..end];
            
            let mut payload = Vec::new();
            payload.push(i as u8);
            payload.push(total_chunks as u8);
            payload.extend_from_slice(chunk_data);
            
            packets.push(Packet {
                version: 0x01,
                type_byte: (msg_type.clone() as u8) | 0x80,
                sender_pubkey: signing_key.verifying_key().to_bytes(),
                timestamp,
                signature: signature.to_bytes(),
                payload_length: payload.len() as u16,
                payload,
            });
        }
        packets
    }
}

/// Replay protection: rolling window of +/- 300 seconds
pub fn is_timestamp_valid(timestamp: u64) -> bool {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if timestamp > now + 300 || timestamp + 300 < now {
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;
    
    #[test]
    fn test_packet_encode_decode() {
        let keypair = generate_keypair();
        let payload = b"Hello, World!";
        let packets = build_packets(&keypair, MsgType::TxGossip, payload);
        assert_eq!(packets.len(), 1);
        
        let encoded = packets[0].encode();
        let decoded = Packet::decode(&encoded).unwrap();
        
        assert_eq!(decoded.version, 0x01);
        assert_eq!(decoded.type_byte, MsgType::TxGossip as u8);
        assert_eq!(decoded.payload, payload);
    }
    
    #[test]
    fn test_fragmentation() {
        let keypair = generate_keypair();
        let payload = vec![0u8; 300]; // Will fragment
        let packets = build_packets(&keypair, MsgType::TxGossip, &payload);
        assert!(packets.len() > 1);
        
        for p in &packets {
            assert!(p.is_fragmented());
            let encoded = p.encode();
            assert!(encoded.len() <= 256);
            let decoded = Packet::decode(&encoded).unwrap();
            assert_eq!(decoded.payload_length as usize, decoded.payload.len());
        }
    }
}

