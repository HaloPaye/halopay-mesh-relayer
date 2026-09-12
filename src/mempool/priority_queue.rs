use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrioritizedTransaction {
    pub tx_id: [u8; 32],
    pub priority: u32,
    pub timestamp: u64,
    pub payload_size: usize,
}

impl Ord for PrioritizedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

impl PartialOrd for PrioritizedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct TransactionPriorityQueue {
    max_items: usize,
    heap: BinaryHeap<PrioritizedTransaction>,
}

impl TransactionPriorityQueue {
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            heap: BinaryHeap::with_capacity(max_items),
        }
    }

    pub fn push(&mut self, item: PrioritizedTransaction) -> bool {
        if self.heap.len() >= self.max_items {
            return false;
        }
        self.heap.push(item);
        true
    }

    pub fn pop(&mut self) -> Option<PrioritizedTransaction> {
        self.heap.pop()
    }

    pub fn prune_older_than(&mut self, current_time: u64, max_age: u64) -> usize {
        let initial_count = self.heap.len();
        let valid_items: Vec<PrioritizedTransaction> = self
            .heap
            .drain()
            .filter(|tx| current_time.saturating_sub(tx.timestamp) <= max_age)
            .collect();
        self.heap = BinaryHeap::from(valid_items);
        initial_count - self.heap.len()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
