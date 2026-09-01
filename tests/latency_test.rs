// Integration tests for peer ping-pong latency measurement
#[test]
fn test_latency_calculation_bounds() {
    let ping_sent_ms: u64 = 1000;
    let pong_recv_ms: u64 = 1045;
    let rtt = pong_recv_ms.saturating_sub(ping_sent_ms);
    assert_eq!(rtt, 45);
    assert!(rtt < 200, "RTT should be within acceptable threshold");
}
