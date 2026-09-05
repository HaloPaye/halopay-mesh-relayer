//! Batched write-ahead log buffer with transaction sequencing and flush triggers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalRecord {
    pub tx_hash: String,
    pub payload: Vec<u8>,
    pub sequence: u64,
}

pub struct WalBuffer {
    records: Vec<WalRecord>,
    capacity: usize,
    sequence_counter: u64,
}

impl WalBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
            sequence_counter: 0,
        }
    }

    pub fn append(&mut self, tx_hash: String, payload: Vec<u8>) -> u64 {
        self.sequence_counter = self.sequence_counter.saturating_add(1);
        self.records.push(WalRecord {
            tx_hash,
            payload,
            sequence: self.sequence_counter,
        });
        self.sequence_counter
    }

    pub fn is_flush_needed(&self) -> bool {
        self.records.len() >= self.capacity
    }

    pub fn flush_batch(&mut self) -> Vec<WalRecord> {
        std::mem::replace(&mut self.records, Vec::with_capacity(self.capacity))
    }

    pub fn pending_count(&self) -> usize {
        self.records.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_buffer_sequencing_and_flush() {
        let mut buffer = WalBuffer::new(2);
        assert_eq!(buffer.pending_count(), 0);
        assert!(!buffer.is_flush_needed());

        let seq1 = buffer.append("tx_1".to_string(), b"payload1".to_vec());
        assert_eq!(seq1, 1);
        assert_eq!(buffer.pending_count(), 1);
        assert!(!buffer.is_flush_needed());

        let seq2 = buffer.append("tx_2".to_string(), b"payload2".to_vec());
        assert_eq!(seq2, 2);
        assert_eq!(buffer.pending_count(), 2);
        assert!(buffer.is_flush_needed());

        let flushed = buffer.flush_batch();
        assert_eq!(flushed.len(), 2);
        assert_eq!(flushed[0].tx_hash, "tx_1");
        assert_eq!(flushed[1].tx_hash, "tx_2");
        assert_eq!(buffer.pending_count(), 0);
        assert!(!buffer.is_flush_needed());
    }
}