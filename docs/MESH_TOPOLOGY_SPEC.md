# HaloPay Mesh Relayer - Libp2p Network Topology

## Architecture Overview
The HaloPay Mesh Relayer forms a decentralized peer-to-peer transport layer utilizing Libp2p GossipSub v1.2 protocols.

### Key Specifications
1. **Topic Hierarchy**:
   - /halopay/tx/v1: Transaction propagation
   - /halopay/heartbeat/v1: Node liveness and scoring
2. **Backpressure & Buffer Bounds**:
   - Max channel capacity: 10,000 messages
   - Eviction strategy: Drop lowest score peer messages on saturation
3. **Peer Scoring & Penalties**:
   - Invalid signature penalty: -500 pts (immediate eviction)
   - Stale timestamp (>60s): -50 pts