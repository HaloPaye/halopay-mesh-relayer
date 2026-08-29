<h1 align="center">HaloPay Mesh Relayer</h1>

<p align="center">
  A robust, asynchronous Rust daemon designed to facilitate offline transaction settlement in humanitarian zones. It forms a resilient mesh network across POS hardware using BLE and local WiFi direct links to route encrypted payloads out of internet-blackout areas.
</p>

<p align="center">
  <a href="https://github.com/HaloPaye/halopay-mesh-relayer/actions"><img alt="CI/CD" src="https://img.shields.io/badge/build-passing-brightgreen"></a>
  <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-yellow.svg">
  <img alt="Rust" src="https://img.shields.io/badge/rust-daemon-blue">
</p>

---

## Core Architecture

The project is structured as an Enterprise Cargo Workspace, splitting concerns into highly optimized and testable crates:

1. **mesh-crypto**: Core cryptographic primitives including Ed25519 signing, BLAKE3 hashing, and ChaCha20 encryption.
2. **mesh-protocol**: Message serialization, binary types, and the highly efficient 256-byte fragmentation protocol.
3. **mesh-storage**: SQLite persistence schema, mempool state management, and LRU caches.
4. **mesh-transport**: Hardware layer abstractions for Bluetooth Low Energy (BLE), LoRa, and Simulation traits.
5. **mesh-node**: The core daemon orchestrator, gossip flood routing logic, and HTTP API settlement client.
6. **mesh-tui**: A \atatui\-based terminal dashboard to visualize network topology and settlement states in real-time.

### Architecture Diagram

\\\mermaid
graph TD
  POS[Offline POS Device] -->|BLE / WiFi Direct| Node[Mesh Node]
  Node -->|Gossip Flood| Node2[Peer Mesh Node]
  Node2 -->|Gossip Flood| Gateway[Internet-Connected Gateway]
  Gateway -->|Encrypted Payload| API[HaloPay Settlement API]
  API -->|Settlement ACK| Gateway
\\\

### The Hard Problem: Offline Double Spending

In humanitarian zones without stable internet, merchants must still accept digital USDC payments securely. However, the inability to verify balances in real-time creates a significant attack vector: offline double-spending.

This relayer solves this problem using a deterministic conflict resolution rule natively executed in the mesh layer:

1. **Cryptographic Signatures**: Every transaction is cryptographically signed using Ed25519 by the merchant's POS device.
2. **Conflict Detection**: If two nodes broadcast conflicting transactions (i.e. same monotonic nonce, same merchant key, but different payloads or signatures), the network mempools instantly detect the anomaly.
3. **Deterministic Resolution**: The rule is strictly mathematical — **The transaction whose Ed25519 signature produces the lowest BLAKE3 hash wins.**
4. **Damage Control**: The losing transaction is dropped entirely across the mesh, and a \SettlementFailed\ ACK is routed back to the offending UI.
5. **Exposure Limits**: All routing guarantees are strictly mathematically bound to max damage isolation limits (e.g., a maximum of 500 USDC total offline exposure per partition).

---

## Tech Stack

- **Core Daemon**: Rust, Tokio (Async Runtime)
- **Cryptography**: Ed25519, BLAKE3, ChaCha20
- **Database**: SQLite (local persistence & mempool)
- **Monitoring**: \atatui\ (Terminal UI)
- **Transport**: BLE, LoRa, WiFi Direct

---

## Setup & Quick Start

A standard \Makefile\ is provided for CI and deployment commands:

\\\ash
# Clone the repository
git clone https://github.com/HaloPaye/halopay-mesh-relayer.git
cd halopay-mesh-relayer

# Build the workspace
make build

# Run unit tests
make test

# Lint and format code
make lint

# Cross-compile for Raspberry Pi / ARM architectures
make cross-compile-arm
\\\

### Running the Simulation Harness

To evaluate the system, we provide a massive simulation harness that bootstraps virtual nodes (A, B, C, D) connected via in-memory broadcast channels, mimicking an unstable physical environment.

\\\ash
# Run the simulation harness and launch the TUI dashboard
make run-sim
# or: cargo run --bin mesh-sim
\\\

The simulation will run various topologies: Partition & Heal, Relaying, Disappearance, Malicious Injections, and Duplicates. The live TUI dashboard will provide a real-time visualization of mempool sizes, active peers, and scrolling gossip/settlement ACKs.

*Note: A \systemd\ service file is included in \init/halopay-mesh.service\ to deploy the daemon onto a Raspberry Pi or other POS hardware seamlessly.*

## Maintainers & Contact

| Maintainer | Contact / Telegram | Role |
| :--- | :--- | :--- |
| HaloPay Team | [@HaloPayDev](https://t.me/HaloPayDev) | Core Protocol Engineering |
| Lead Engineer | security@halopay.io | Security & Operations |

## Contributors

[![Contributors](https://contrib.rocks/image?repo=HaloPaye/halopay-mesh-relayer)](https://github.com/HaloPaye/halopay-mesh-relayer/graphs/contributors)

---

## License

This project is licensed under the MIT License.
