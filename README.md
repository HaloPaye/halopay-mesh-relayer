# HaloPay Mesh Relayer

HaloPay Mesh Relayer is a robust, asynchronous rust daemon designed to facilitate offline transaction settlement in humanitarian zones. It forms a resilient mesh network across commercially available POS hardware using Bluetooth Low Energy (BLE) and local WiFi direct links, aggressively routing encrypted payloads out of internet-blackout areas.

## The Hard Problem: Offline Double Spending

In humanitarian zones without stable internet, merchants must still accept digital USDC payments securely. However, the inability to verify balances in real-time creates a significant attack vector: offline double-spending.

This relayer solves this problem using a deterministic conflict resolution rule natively executed in the mesh layer:
- Every transaction is signed using Ed25519.
- If two nodes broadcast conflicting transactions (same monotonic nonce, same merchant key, different payloads/signatures), they are immediately detected.
- The rule: **The transaction whose Ed25519 signature produces the lowest BLAKE3 hash wins.** 
- The losing transaction is dropped entirely and a `SettlementFailed` ACK is routed back to the offending UI.
- All routing guarantees are strictly mathematically bound to max damage isolation limits (e.g., 500 USDC total offline exposure per partition).

## Architecture

The project is structured as an Enterprise Cargo Workspace:
- `mesh-crypto`: Core cryptographic primitives (Ed25519 signing, BLAKE3 hashing, ChaCha20 encryption).
- `mesh-protocol`: Message serialization, binary types, and the 256-byte fragmentation protocol.
- `mesh-storage`: SQLite persistence schema, mempool state, and LRU caches.
- `mesh-transport`: BLE, LoRa, and Simulation trait abstractions.
- `mesh-node`: The core daemon, gossip flood routing logic, and HTTP API settlement client.
- `mesh-tui`: A ratatui-based terminal dashboard to visualize network topology and settlement states.

## Running the Simulation Harness

To evaluate the system, we provide a massive simulation harness that bootstraps virtual nodes (A, B, C, D) connected via in-memory broadcast channels, mimicking an unstable physical environment.

1. `make run-sim` (or `cargo run --bin mesh-sim`)
2. The simulation will run various topologies: Partition & Heal, Relaying, Disappearance, Malicious Injections, and Duplicates.
3. A live TUI dashboard will automatically open, providing a real-time visualization of mempool sizes, active peers, and scrolling gossip/settlement ACKs.

## Building and Deployment

A standard `Makefile` is provided for CI and deployment commands:
- `make build`
- `make test`
- `make lint`
- `make cross-compile-arm`

A `systemd` service file is included in `init/halopay-mesh.service` to deploy the daemon onto a Raspberry Pi or other POS hardware seamlessly.
