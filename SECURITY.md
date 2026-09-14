# Security Policy

HaloPay takes the security of its decentralized mesh protocol, cryptographic primitives, and offline transaction rails extremely seriously.

---

## Supported Versions

Only the latest commit on the `main` branch is actively maintained and patched for security vulnerabilities.

| Version | Supported          |
| :------ | :----------------- |
| `main`  | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Reporting a Vulnerability

If you discover a vulnerability, cryptographic flaw, or double-spend vector within the HaloPay Mesh Relayer, **please DO NOT open a public GitHub issue or pull request.**

Please report all security vulnerabilities via responsible disclosure:

* **Email:** [security@halopay.io](mailto:security@halopay.io)
* **Response Time:** Our security team will acknowledge receipt of your report within **24 hours**.
* **Patch Timeline:** We aim to triage and release a security patch within **48–72 hours** of confirmation.

### What to Include in Your Report
To help us triage and resolve the issue quickly, please provide:
1. A clear description of the vulnerability and its potential impact.
2. Step-by-step reproduction steps or a minimal proof-of-concept (PoC).
3. The specific affected crate(s) (e.g., `crates/mesh-crypto`, `crates/mesh-protocol`).
4. Any proposed fixes or remediations if available.

---

## Protocol Security Guarantees & Safeguards

The HaloPay Mesh Relayer implements multiple defensive layers to protect merchants and customers during offline network partitions:

1. **Ed25519 Cryptographic Signatures:** Every transaction circulating on the mesh is signed by the originating device. Signatures are validated before packets are added to the local mempool.
2. **Deterministic Conflict Resolution:** To prevent offline double-spending without a central coordinator, the protocol enforces deterministic ordering: **the signature yielding the lowest BLAKE3 hash wins**, and conflicting nonces are automatically evicted.
3. **Mempool Replay Protection:** Monotonic nonces and transaction hash digests prevent replay attacks across partition heal cycles.
4. **Exposure Caps:** Mesh nodes enforce configurable maximum damage isolation caps (default: 500 USDC total offline exposure per partition).

