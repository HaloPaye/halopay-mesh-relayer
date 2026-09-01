# Malicious Peer Ban-List Architecture

## Overview
In a decentralized, low-bandwidth wireless mesh relayer network such as the HaloPay Mesh Relayer, nodes must defend against Sybil attacks, malicious replay vectors, malformed transaction flooding, double-spend broadcasts, and denial-of-service (DoS) attempts. 

The **Malicious Peer Ban-List** subsystem provides a zero-trust, defense-in-depth framework designed to identify, penalize, evict, and block malicious peers dynamically while minimizing overhead on constrained devices.

---

## Ban-List Architecture

The ban-list subsystem operates as a layered filtering pipeline integrated directly into the node's transport and gossip loops:

```
[Inbound Connection / Packet]
              │
              ▼
    ┌───────────────────┐
    │ Connection Gater  │ ── (Is Peer ID or IP in Ban-List?) ──► [Immediate Reject / Drop]
    └───────────────────┘
              │ (Allowed)
              ▼
    ┌───────────────────┐
    │ Protocol Decoding │ ── (Malformed Header / Bad Magic) ──► [Reputation Penalty (-50)]
    └───────────────────┘
              │ (Valid Frame)
              ▼
    ┌───────────────────┐
    │ Crypto & Validity │ ── (Bad Signature / Double Spend) ──► [Reputation Penalty (-100)]
    └───────────────────┘
              │ (Verified)
              ▼
    ┌───────────────────┐
    │  Reputation Engine│ ── (Score < Threshold) ─────────────► [Trigger Automated Eviction & Ban]
    └───────────────────┘
              │ (Healthy)
              ▼
    [Mempool / Broadcast]
```

### Storage Layers
1. **In-Memory Cache (Fast Path)**:
   - A concurrent hash set (`DashMap<PeerId, BanRecord>`) backed by an LRU eviction buffer for fast O(1) lookups during packet reception and connection handshakes.
2. **Persistent Storage (SQLite Database)**:
   - Durable ledger of banned peer identities, ban reasons, initial timestamps, and expiration times.
   - Automatically synchronizes across restarts to prevent banned peers from re-connecting upon node reboot.

---

## Peer Reputation Scoring

The Reputation Scoring Engine evaluates every connected peer based on deterministic operational metrics. Nodes maintain a real-time reputation score S in [-100, +100] initialized to S_0 = 0 for unknown peers.

### Scoring Table

| Event / Behavior | Score Delta (ΔS) | Classification | Action |
| :--- | :--- | :--- | :--- |
| **Valid Transaction Relay** | +2 | Positive Signal | Increment score up to +100 cap |
| **Timely Heartbeat Response (Ping < 200ms)** | +1 | Positive Signal | Maintain active health status |
| **Signature Verification Failure** | -40 | Major Infraction | Immediate verification failure log |
| **Double-Spend Attempt Broadcast** | -100 | Critical Malice | Immediate permanent ban |
| **Malformed / Invalid Packet Encoding** | -25 | Protocol Violation | Drops packet, degrades peer score |
| **Replay / Stale Timestamp Flood** | -30 | DoS Vector | Rate limits peer |
| **Exceeding Offline Exposure Limit** | -20 | Policy Violation | Drops transaction |

### Score Decay and Forgiveness
- Scores slowly decay toward the baseline (0) over a configurable epoch window (e.g., 10 minutes).
- Positive reputation ensures high-quality peers maintain preferential gossip relay slots.
- Negative reputation compounds rapidly upon repeated infractions.

---

## Automated Eviction Policies

When a peer's score drops below predefined critical thresholds, the node automatically executes eviction protocols:

```
          Reputation Score:
  +100 ───────────────────────────── (Trusted Peer)
     0 ───────────────────────────── (Neutral / New Peer)
   -30 ───────────────────────────── (Probationary Threshold: Rate-limited)
   -60 ───────────────────────────── (Temporary Ban Threshold: Disconnect + 1 hr Cooldown)
  -100 ───────────────────────────── (Permanent Ban Threshold: Blacklisted permanently)
```

1. **Probation Mode (Score <= -30)**:
   - Peer bandwidth allocation throttled by 75%.
   - Non-critical messages (e.g., Hello, low-priority gossip) are dropped.
2. **Temporary Ban (Score <= -60)**:
   - Connection is immediately severed with `ConnectionClose`.
   - Peer is banned for a progressive cooling-off period (e.g., 15 mins -> 1 hr -> 24 hrs).
3. **Permanent Blacklist (Score <= -100)**:
   - Hard eviction: Peer public key and network endpoints are written to the persistent blacklist table.
   - Any future connection attempts are rejected at the transport layer before cryptographic handshakes occur.

---

## Libp2p Connection Gating

The HaloPay Mesh Relayer employs `libp2p::connection_limits` and custom `ConnectionGating` handlers at the swarm level:

- **Inbound Connection Handshake**: The swarm checks the remote `PeerId` against the in-memory ban set before allocating transport resources (Noise/TLS handshake).
- **Multiaddr & Subnet Gating**: In addition to public keys, IP subnets generating coordinated Sybil floods are blocked at the socket listener level.
- **Dial Interception**: Outbound gossip routing engines ignore peers tagged with active bans or probationary scores.

---

## Telemetry & Observability

All ban-list actions, reputation mutations, and connection drops emit structured telemetry events to ensure full visibility:

### Structured JSON Logs
```json
{
  "timestamp": 1725178800,
  "node_id": "a1b2c3d4",
  "event": "PEER_BANNED",
  "peer_id": "12D3KooW...",
  "reason": "DOUBLE_SPEND_ATTEMPT",
  "score": -100,
  "ban_duration_secs": 86400
}
```

### Prometheus Metrics
- `halopay_mesh_banned_peers_total{reason="invalid_sig|double_spend|dos"}`: Counter tracking total ban events.
- `halopay_mesh_active_banned_peers`: Gauge of currently blacklisted peers.
- `halopay_mesh_peer_reputation_score{peer_id="..."}`: Gauge tracking real-time peer scores.
- `halopay_mesh_gated_connections_rejected_total`: Counter for connections dropped at the swarm level.
