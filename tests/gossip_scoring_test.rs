#[test]
fn test_gossip_score_decay_calculation() {
    let initial_score = 100.0;
    let decay_factor = 0.9;
    let decayed = initial_score * decay_factor;
    assert!((decayed - 90.0).abs() < f64::EPSILON);
}