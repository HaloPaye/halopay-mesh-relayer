//! Peer discovery heartbeat monitor with adaptive timeout heuristics.

#[derive(Debug, Clone)]
pub struct HeartbeatMonitor {
    heartbeat_interval_ms: u64,
    timeout_window_ms: u64,
}

impl HeartbeatMonitor {
    pub fn new(heartbeat_interval_ms: u64, timeout_window_ms: u64) -> Self {
        Self {
            heartbeat_interval_ms,
            timeout_window_ms: timeout_window_ms.max(heartbeat_interval_ms.saturating_add(1)),
        }
    }

    pub fn is_peer_alive(&self, last_seen_timestamp: u64, current_timestamp: u64) -> bool {
        let elapsed = current_timestamp.saturating_sub(last_seen_timestamp);
        elapsed < self.timeout_window_ms
    }

    pub fn calculate_next_ping_deadline(&self, last_ping_timestamp: u64) -> u64 {
        last_ping_timestamp.saturating_add(self.heartbeat_interval_ms)
    }

    pub fn interval_ms(&self) -> u64 {
        self.heartbeat_interval_ms
    }

    pub fn timeout_ms(&self) -> u64 {
        self.timeout_window_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_alive_and_deadline() {
        let monitor = HeartbeatMonitor::new(1000, 3000);
        assert!(monitor.is_peer_alive(5000, 6000));
        assert!(monitor.is_peer_alive(5000, 7999));
        assert!(!monitor.is_peer_alive(5000, 8000));
        assert_eq!(monitor.calculate_next_ping_deadline(5000), 6000);
    }
}