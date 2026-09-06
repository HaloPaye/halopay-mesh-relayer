//! Bounded event bus dispatcher with backpressure protection and capacity monitoring.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshEvent {
    pub topic: String,
    pub payload: Vec<u8>,
    pub priority: u8,
}

pub struct BoundedEventDispatcher {
    capacity: usize,
    queue: Vec<MeshEvent>,
    dropped_count: u64,
}

impl BoundedEventDispatcher {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            queue: Vec::with_capacity(capacity),
            dropped_count: 0,
        }
    }

    pub fn dispatch(&mut self, event: MeshEvent) -> bool {
        if self.queue.len() >= self.capacity {
            self.dropped_count = self.dropped_count.saturating_add(1);
            false
        } else {
            self.queue.push(event);
            true
        }
    }

    pub fn pop_next(&mut self) -> Option<MeshEvent> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn pending_len(&self) -> usize {
        self.queue.len()
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped_count
    }

    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_dispatcher_backpressure() {
        let mut dispatcher = BoundedEventDispatcher::new(2);
        let ev1 = MeshEvent { topic: "tx".into(), payload: vec![1], priority: 1 };
        let ev2 = MeshEvent { topic: "tx".into(), payload: vec![2], priority: 1 };
        let ev3 = MeshEvent { topic: "tx".into(), payload: vec![3], priority: 1 };

        assert!(dispatcher.dispatch(ev1.clone()));
        assert!(dispatcher.dispatch(ev2.clone()));
        assert!(!dispatcher.dispatch(ev3)); // dropped

        assert_eq!(dispatcher.pending_len(), 2);
        assert_eq!(dispatcher.dropped_count(), 1);
        assert!(dispatcher.is_full());

        let popped = dispatcher.pop_next().unwrap();
        assert_eq!(popped.payload, vec![1]);
        assert_eq!(dispatcher.pending_len(), 1);
    }
}
