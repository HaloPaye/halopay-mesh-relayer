//! Resilient Libp2p stream listener handler with connection reset tracking.

#[derive(Debug, PartialEq, Eq)]
pub enum StreamState {
    Active,
    Reset,
    Closed,
}

pub struct StreamListenerManager {
    peer_id: String,
    state: StreamState,
    reset_count: u32,
    base_backoff_ms: u64,
}

impl StreamListenerManager {
    pub fn new(peer_id: String, base_backoff_ms: u64) -> Self {
        Self {
            peer_id,
            state: StreamState::Active,
            reset_count: 0,
            base_backoff_ms: base_backoff_ms.max(100),
        }
    }

    pub fn handle_stream_reset(&mut self) -> u64 {
        self.state = StreamState::Reset;
        self.reset_count = self.reset_count.saturating_add(1);
        let multiplier = 1u64.checked_shl(self.reset_count.min(6)).unwrap_or(64);
        (self.base_backoff_ms.saturating_mul(multiplier)).min(10_000)
    }

    pub fn mark_reconnected(&mut self) {
        self.state = StreamState::Active;
        self.reset_count = 0;
    }

    pub fn is_active(&self) -> bool {
        self.state == StreamState::Active
    }

    pub fn reset_count(&self) -> u32 {
        self.reset_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_reset_exponential_backoff() {
        let mut manager = StreamListenerManager::new("peer_123".into(), 200);
        assert!(manager.is_active());

        let delay1 = manager.handle_stream_reset();
        assert_eq!(delay1, 400); // 200 * 2^1
        assert_eq!(manager.reset_count(), 1);

        let delay2 = manager.handle_stream_reset();
        assert_eq!(delay2, 800); // 200 * 2^2

        manager.mark_reconnected();
        assert!(manager.is_active());
        assert_eq!(manager.reset_count(), 0);
    }
}
