//! High-performance malicious peer ban-list with TTL expiration and bounded memory.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BanEntry {
    pub banned_until: Instant,
    pub reason: String,
    pub violation_count: u32,
}

pub struct PeerBanList {
    capacity: usize,
    default_ban_duration: Duration,
    bans: HashMap<String, BanEntry>,
}

impl PeerBanList {
    pub fn new(capacity: usize, default_ban_secs: u64) -> Self {
        Self {
            capacity: capacity.max(16),
            default_ban_duration: Duration::from_secs(default_ban_secs.max(1)),
            bans: HashMap::with_capacity(capacity.min(256)),
        }
    }

    pub fn ban_peer(&mut self, peer_id: &str, reason: &str) {
        let now = Instant::now();
        self.prune_expired(now);

        if self.bans.len() >= self.capacity {
            // Evict the entry with the earliest expiration to bound memory
            if let Some(earliest_key) = self
                .bans
                .iter()
                .min_by_key(|(_, entry)| entry.banned_until)
                .map(|(k, _)| k.clone())
            {
                self.bans.remove(&earliest_key);
            }
        }

        let entry = self
            .bans
            .entry(peer_id.to_string())
            .and_modify(|e| {
                e.banned_until = now + self.default_ban_duration;
                e.violation_count = e.violation_count.saturating_add(1);
                e.reason = reason.to_string();
            })
            .or_insert(BanEntry {
                banned_until: now + self.default_ban_duration,
                reason: reason.to_string(),
                violation_count: 1,
            });

        let _ = entry;
    }

    pub fn is_banned(&mut self, peer_id: &str) -> bool {
        let now = Instant::now();
        if let Some(entry) = self.bans.get(peer_id) {
            if entry.banned_until > now {
                return true;
            }
        }
        self.bans.remove(peer_id);
        false
    }

    pub fn unban(&mut self, peer_id: &str) -> bool {
        self.bans.remove(peer_id).is_some()
    }

    pub fn prune_expired(&mut self, now: Instant) {
        self.bans.retain(|_, entry| entry.banned_until > now);
    }

    pub fn active_ban_count(&self) -> usize {
        self.bans.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_ban_and_lookup() {
        let mut list = PeerBanList::new(10, 60);
        assert!(!list.is_banned("peer_alpha"));

        list.ban_peer("peer_alpha", "invalid signature spam");
        assert!(list.is_banned("peer_alpha"));
        assert_eq!(list.active_ban_count(), 1);

        list.unban("peer_alpha");
        assert!(!list.is_banned("peer_alpha"));
    }

    #[test]
    fn test_capacity_bounding() {
        let mut list = PeerBanList::new(2, 60);
        list.ban_peer("peer_1", "reason 1");
        list.ban_peer("peer_2", "reason 2");
        list.ban_peer("peer_3", "reason 3");

        assert_eq!(list.active_ban_count(), 2);
        assert!(list.is_banned("peer_3"));
    }
}
