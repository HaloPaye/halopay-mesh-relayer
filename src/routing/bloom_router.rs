//! Accelerated duplicate message filtering using a counting bloom filter cache.

pub struct BloomRouterFilter {
    slots: Vec<u8>,
    num_hashes: usize,
}

impl BloomRouterFilter {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self {
            slots: vec![0; size.max(16)],
            num_hashes: num_hashes.clamp(1, 4),
        }
    }

    fn hash(&self, key: &str, seed: usize) -> usize {
        let mut h: u64 = 0x811c9dc5;
        for b in key.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x01000193 ^ (seed as u64));
        }
        (h as usize) % self.slots.len()
    }

    pub fn insert_and_check_seen(&mut self, key: &str) -> bool {
        let mut all_seen = true;
        for i in 0..self.num_hashes {
            let idx = self.hash(key, i);
            if self.slots[idx] == 0 {
                all_seen = false;
            }
            self.slots[idx] = self.slots[idx].saturating_add(1);
        }
        all_seen
    }

    pub fn reset(&mut self) {
        self.slots.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_router_detection() {
        let mut filter = BloomRouterFilter::new(64, 3);
        assert!(!filter.insert_and_check_seen("msg_alpha"));
        assert!(filter.insert_and_check_seen("msg_alpha")); // already seen
        assert!(!filter.insert_and_check_seen("msg_beta"));
    }
}
