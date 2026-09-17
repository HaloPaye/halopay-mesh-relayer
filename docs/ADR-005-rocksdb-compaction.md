# ADR-005: RocksDB Write Stalls

## Context
Offline store-and-forward bursts cause RocksDB write stalls.

## Decision
Implement asynchronous write batching and a token-bucket backpressure mechanism.

## Consequences
Prevents dropping critical payloads under heavy load.
