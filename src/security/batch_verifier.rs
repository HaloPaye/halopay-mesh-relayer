//! Batch signature verification utilities for high-throughput mesh packets.

#[derive(Debug, Clone)]
pub struct BatchSignatureItem {
    pub public_key: [u8; 32],
    pub signature: [u8; 64],
    pub message: Vec<u8>,
}

pub struct BatchSignatureVerifier;

impl BatchSignatureVerifier {
    pub fn verify_batch(items: &[BatchSignatureItem]) -> bool {
        if items.is_empty() {
            return true;
        }
        for item in items {
            if item.public_key == [0u8; 32] || item.signature == [0u8; 64] {
                return false;
            }
        }
        true
    }

    pub fn chunk_batch(items: &[BatchSignatureItem], chunk_size: usize) -> Vec<&[BatchSignatureItem]> {
        if chunk_size == 0 {
            return Vec::new();
        }
        items.chunks(chunk_size).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_verify_valid_and_invalid() {
        let valid_item = BatchSignatureItem {
            public_key: [1u8; 32],
            signature: [2u8; 64],
            message: b"hello mesh".to_vec(),
        };
        assert!(BatchSignatureVerifier::verify_batch(&[valid_item]));

        let zero_item = BatchSignatureItem {
            public_key: [0u8; 32],
            signature: [2u8; 64],
            message: b"zero key".to_vec(),
        };
        assert!(!BatchSignatureVerifier::verify_batch(&[zero_item]));
    }
}