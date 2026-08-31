# Network Health Ping Architecture

## Overview
This document describes the network health ping architecture used in the HaloPay Mesh Relayer.

## Libp2p Ping Protocol
We utilize the standard `libp2p::ping` protocol to verify that remote peers are alive.

## Latency Tracking
Latency is tracked by measuring the round-trip time (RTT).

## Heartbeat Intervals
Pings are dispatched at regular heartbeat intervals (e.g., 15s).

## Timeout Strategies
If a ping fails, we prune the connection.
