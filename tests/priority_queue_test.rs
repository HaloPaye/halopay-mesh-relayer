#[test]
fn test_priority_ordering_logic() {
    let mut priorities = vec![10u32, 50u32, 20u32, 100u32];
    priorities.sort_by(|a, b| b.cmp(a));
    assert_eq!(priorities[0], 100);
    assert_eq!(priorities[1], 50);
}

#[test]
fn test_mempool_pruning_calculation() {
    let current_time: u64 = 1000;
    let max_age: u64 = 300;
    let tx1_time: u64 = 800;
    let tx2_time: u64 = 600;

    let is_tx1_valid = current_time.saturating_sub(tx1_time) <= max_age;
    let is_tx2_valid = current_time.saturating_sub(tx2_time) <= max_age;

    assert!(is_tx1_valid);
    assert!(!is_tx2_valid);
}
