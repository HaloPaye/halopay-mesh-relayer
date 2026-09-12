pub struct BatchDigestCalculator;

impl BatchDigestCalculator {
    pub fn compute_batch_digest(tx_hashes: &[[u8; 32]]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        for h in tx_hashes {
            hasher.update(h);
        }
        *hasher.finalize().as_bytes()
    }
}
