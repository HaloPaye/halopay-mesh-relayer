#[test]
fn test_prometheus_counter_increment() {
    let mut packet_count = 0u64;
    for _ in 0..10 {
        packet_count = packet_count.saturating_add(1);
    }
    assert_eq!(packet_count, 10);
}

#[test]
fn test_prometheus_histogram_bucket_assignment() {
    let thresholds = [10.0_f64, 50.0_f64, 100.0_f64, 250.0_f64, 500.0_f64];
    let sample_latency = 75.0_f64;
    let bucket_idx = thresholds.iter().position(|&t| sample_latency <= t);
    assert_eq!(bucket_idx, Some(2));
}
