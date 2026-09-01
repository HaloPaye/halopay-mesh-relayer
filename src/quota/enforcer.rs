// Per-peer bandwidth quota and token bucket rate limiter
pub struct BandwidthQuota {
    pub peer_id: String,
    pub bytes_per_sec_limit: u64,
}

impl BandwidthQuota {
    pub fn check_allowance(&self, requested_bytes: u64) -> bool {
        requested_bytes <= self.bytes_per_sec_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_check() {
        let q = BandwidthQuota { peer_id: "peer_1".into(), bytes_per_sec_limit: 1048576 };
        assert!(q.check_allowance(500000));
        assert!(!q.check_allowance(2000000));
    }
}
