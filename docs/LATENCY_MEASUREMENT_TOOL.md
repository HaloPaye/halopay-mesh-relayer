# Latency Measurement Tool Specification

## Overview
In a decentralized crisis payment mesh, peer propagation latency directly impacts consensus speed, transaction gossiping efficiency, and settlement finality. The **Latency Measurement Tool** (`LatencyTracker`) provides high-resolution, round-trip time (RTT) tracking across connected Libp2p peers and Bluetooth Low Energy (BLE) peripheral relays.

---

## Architectural Design

```
+------------------+         Ping Payload (Nonce + Timestamp)        +------------------+
|                  | ---------------------------------------------> |                  |
|    Local Peer    |                                                |   Remote Peer    |
|   LatencyProbe   | <--------------------------------------------- |  Pong Responder  |
|                  |          Pong Response (Nonce Echoed)          +------------------+
+------------------+
        |
        v
+------------------+
|   Jitter Filter  | -> Outlier Clamping (IQR / Robust EWMA)
+------------------+
        |
        v
+------------------+
| Routing Weight   | -> Prioritize Lowest-Latency Overlay Links
+------------------+
```

### 1. Probing Mechanics
* **Heartbeat Ping:** Probes are periodically dispatched (default: every `5000ms`) using lightweight 32-byte nonces.
* **Timestamp Echo:** Ping frames record the local high-resolution clock (`std::time::Instant`). Upon receiving the corresponding pong frame with matching nonce, RTT is recorded.
* **Exponential Moving Average (EWMA):**
  $$\text{RTT}_{\text{smoothed}} = \alpha \cdot \text{RTT}_{\text{new}} + (1 - \alpha) \cdot \text{RTT}_{\text{prev}}$$
  Where $\alpha = 0.2$ to dampen transient network spikes.

---

## Outlier Rejection & Jitter Protection
Network jitter and cellular packet re-transmissions can distort raw RTT measurements. The latency monitor applies two protective guards:
1. **Upper Threshold Clamping:** Any RTT exceeding `10,000ms` triggers a timeout event and increases peer disconnection probability.
2. **Standard Deviation Window:** Samples outside $\pm 2.5\sigma$ within a sliding 20-sample window are marked as transient anomalies and excluded from the routing calculation.

---

## Integration with Routing & Gossip
* **Gossipsub Score Adjustments:** Low-latency peers receive a score boost in `BloomRouter` and gossip mesh topic fan-out.
* **Fallback Degradation:** If a peer's smoothed latency exceeds `2500ms`, the relayer switches from aggressive flood-gossip to targeted unicast gossip to conserve wireless bandwidth.

---

## Telemetry & Metrics
The latency measurement subsystem exports Prometheus gauges:
* `halopay_mesh_peer_rtt_ms{peer_id="..."}`: Current smoothed RTT in milliseconds.
* `halopay_mesh_ping_timeouts_total{peer_id="..."}`: Cumulative missed pong heartbeats.
* `halopay_mesh_jitter_variance{peer_id="..."}`: Rolling jitter variance.
