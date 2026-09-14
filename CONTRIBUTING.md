# Contributing to HaloPay Mesh Relayer

Thank you for your interest in contributing to the **HaloPay Mesh Relayer**! This repository provides an asynchronous Rust daemon and cryptographic gossip protocol for offline transaction routing in humanitarian crisis zones and emerging markets.

To ensure stability, code quality, and smooth collaboration, please follow our contribution guidelines below.

---

## 🚀 How to Contribute

### 1. Select an Open Issue
* Check the repository's open issues labeled `status: ready-for-dev`, `good first issue`, or `help wanted`.
* Read through the issue's **Background**, **Technical Scope**, and **Acceptance Criteria**.

### 2. Request Assignment Before Starting Work
* **Please do not start coding or open unsolicited pull requests without being assigned.** This prevents multiple contributors from duplicating effort on the same task.
* Comment on the issue describing your proposed approach or requesting assignment. A maintainer will triage and assign the issue to you.

### 3. Branching Strategy
* Create your feature branch off `main`:
  ```bash
  git checkout -b feat/issue-<issue_number>-<short-description>
  ```
  *Examples:*
  * `feat/issue-42-chacha20-encryption`
  * `fix/issue-55-mempool-ttl-prune`
  * `docs/issue-60-architecture-spec`

### 4. Pull Request Standards
* **Title Format:** PR titles MUST follow the format:
  ```text
  [#<issue_number>] <Imperative description of change>
  ```
  *Example:* `[#42] Implement ChaCha20-Poly1305 symmetric packet payload encryption`
* **Issue Linking:** In your PR description, explicitly link the issue:
  ```text
  Closes #<issue_number>
  ```
* **Atomicity:** Keep each PR focused strictly on the assigned issue. Avoid mixing unrelated refactors or dependency updates into a functional PR.

---

## 🛠️ Local Development & Quality Gates

All pull requests trigger our continuous integration (CI) workflow. Before submitting a PR, ensure all checks pass locally:

### 1. Formatting
```bash
cargo fmt --all -- --check
```

### 2. Static Analysis & Linting
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### 3. Running Unit & Integration Tests
```bash
cargo test --workspace --all-targets
```

### 4. Running the Simulation Harness
```bash
cargo run --bin mesh-sim
# or: make run-sim
```

---

## 🏛️ Repository Architecture

When contributing code, adhere to our multi-crate workspace separation of concerns:

* `crates/mesh-crypto/`: Low-level cryptographic primitives (Ed25519 signing, BLAKE3 deterministic hashing, ChaCha20-Poly1305 encryption).
* `crates/mesh-protocol/`: Wire protocol framing, binary serialization, and packet fragmentation.
* `crates/mesh-storage/`: Local persistence engine, SQLite schema, mempool state, and cache expiration.
* `crates/mesh-transport/`: Hardware transport traits and drivers (Bluetooth Low Energy, LoRa, UDP simulation).
* `crates/mesh-node/`: Core daemon runtime, gossip flood propagation, routing policies, and settlement gateways.
* `crates/mesh-tui/`: Real-time `ratatui` terminal monitoring dashboard.

---

## 📜 Code of Conduct & Licensing

* **Respect & Professionalism:** Treat all contributors, reviewers, and maintainers with respect and courtesy.
* **Licensing:** All contributions to this repository are licensed under the **Apache License, Version 2.0**. By submitting a pull request, you agree that your work will be covered under this license.

