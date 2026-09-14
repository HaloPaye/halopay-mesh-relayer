//! Unit tests for token bucket bandwidth quota enforcer.

pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
        }
    }

    pub fn consume(&mut self, amount: f64) -> bool {
        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }

    pub fn refill(&mut self, elapsed_secs: f64) {
        self.tokens = (self.tokens + self.refill_rate * elapsed_secs).min(self.capacity);
    }

    pub fn tokens(&self) -> f64 {
        self.tokens
    }
}

#[test]
fn test_token_bucket_consumption_and_refill() {
    let mut bucket = TokenBucket::new(100.0, 10.0);
    assert!(bucket.consume(80.0));
    assert_eq!(bucket.tokens(), 20.0);
    assert!(!bucket.consume(30.0)); // exceeds current tokens

    bucket.refill(2.0); // +20 tokens
    assert_eq!(bucket.tokens(), 40.0);
    assert!(bucket.consume(30.0));
    assert_eq!(bucket.tokens(), 10.0);
}
