use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PeerRateLimiter {
    message_counts: HashMap<String, u32>,
    max_rate: u32,
}

impl PeerRateLimiter {
    pub fn new(max_rate: u32) -> Self {
        Self {
            message_counts: HashMap::new(),
            max_rate,
        }
    }

    pub fn allow_message(&mut self, peer_id: &str) -> bool {
        let count = self.message_counts.entry(peer_id.to_string()).or_insert(0);
        if *count < self.max_rate {
            *count += 1;
            true
        } else {
            false
        }
    }
}