---
title: "Design a Ride-Sharing Service (e.g., Uber)"
tags: [geospatial, real-time, matching, websockets, payments, location-tracking]
difficulty: hard
estimated_time: 60min
companies: [Uber, Lyft, Amazon, Apple, DoorDash]
---

## Problem Statement

Design a ride-sharing platform where riders can request rides and nearby drivers
are matched, with real-time location tracking throughout the trip.

## Clarifying Questions

Ask these before designing:

- Scale? (Assume 10M rides/day, 1M concurrent drivers broadcasting location)
- Matching radius? (Assume ~5 km, configurable)
- ETA accuracy required? (Best-effort; within ±1 min acceptable for MVP)
- Payments in scope? (High-level only — not full payment system design)
- Surge pricing? (Mention as a concept, don't fully design)

## Capacity Estimates

  Location updates: 1M active drivers * 1 update/4sec = 250,000 writes/sec
  Ride requests:    10M/day = ~115 req/sec (bursty at commute hours)
  Matching latency: must complete in < 5 seconds end-to-end

## High-Level Design

```
[Rider App]                  [Driver App]
     |                            |
     | Request ride               | Location update (every 4s)
     v                            v
[API Gateway / Load Balancer]
     |           |
     v           v
[Ride Service] [Location Service]
     |               |
     v               v
[Matching Engine] [Location DB (geospatial)]
     |               (Redis GEO / PostGIS)
     v
[Trip Service] ──> [Notification Service]
     |
     v
[Payments Service]
```

## Location Service

The highest-write component. Handles 250K updates/sec.

**Storage: Redis GEO commands**
  - `GEOADD drivers_online <lng> <lat> <driver_id>` on each update
  - `GEORADIUS drivers_online <lng> <lat> 5 km` for nearby driver lookup
  - Redis GEO uses a geohash under the hood (sorted set with score = geohash)
  - In-memory → sub-millisecond reads; perfect for real-time queries

**Geohash alternative for interviews:**
  - Divide world into a grid of cells at fixed precision (e.g., 6 chars ≈ 1.2km²)
  - Driver's location encoded to geohash cell; store set of drivers per cell
  - Query = check current cell + 8 neighboring cells for nearby drivers

Driver location updates are writes to Redis — durability is not critical (stale
location is replaced in 4 seconds). Append to a time-series DB (InfluxDB) only
if historical tracking is needed.

## Matching Engine

When a rider requests a ride:
1. Look up rider location → compute geohash or GEORADIUS
2. Find available drivers within radius, sorted by distance
3. Send push offer to top N candidates simultaneously (not sequentially)
4. First driver to accept → matched; others receive cancellation
5. If no acceptance within timeout → expand radius and retry

**State machine for a ride:**
  REQUESTED → MATCHING → ACCEPTED → DRIVER_EN_ROUTE → ARRIVED
  → IN_TRIP → COMPLETED → (RATED)

State stored in a relational DB (Postgres) for durability. Transitions trigger
events to Notification Service and Payments Service.

## Real-Time Communication

Both rider and driver need live updates:

- **Driver → Server:** HTTP polling every 4s is simple but wasteful
  Better: persistent WebSocket or long-polling for driver location streaming
- **Server → Rider/Driver:** push updates (match found, driver location, ETA)
  Use WebSocket connections managed by a dedicated Presence Service
  Or: server-sent events (SSE) for simpler one-way push

At scale, WebSocket connections are stateful — use a connection registry
(Redis key: user_id → server_node) to route server-originated pushes to the
correct node holding the open connection.

## Trip Service & Database

  trips
  ┌─────────────────┬────────────────────────────────┐
  │ trip_id         │ UUID        PRIMARY KEY         │
  │ rider_id        │ UUID                            │
  │ driver_id       │ UUID        NULLABLE            │
  │ status          │ ENUM(requested, ..., completed) │
  │ pickup_lat/lng  │ FLOAT                           │
  │ dropoff_lat/lng │ FLOAT       NULLABLE            │
  │ requested_at    │ TIMESTAMP                       │
  │ fare_cents      │ INT         NULLABLE            │
  └─────────────────┴────────────────────────────────┘

Write to Postgres with read replicas for trip history queries.

## Deep Dives

### ETA Calculation
  - Use a road graph (OpenStreetMap data + Dijkstra/A*) for routing
  - In practice, use a maps API or proprietary routing engine
  - Factor in real-time traffic (ingest traffic events, adjust edge weights)
  - Update ETA every ~30s during the trip

### Surge Pricing
  - Monitor supply (available drivers) vs demand (open requests) per geohash zone
  - When demand/supply ratio exceeds threshold → apply multiplier
  - Simple rule: if queue_length > available_drivers * 2 → surge = 1.5×
  - Communicate surge to riders before confirmation

### Driver Dispatch at Scale
  - Single region: Redis GEO handles millions of drivers
  - Multi-region: shard by geographic region (US East, US West, EU, etc.)
  - Each region is an independent cluster; cross-region trips are rare

### Payments
  - Fare calculated server-side (distance × rate + base fare)
  - Charge rider after trip completion via payment provider
  - Idempotency keys prevent double-charges on retry
  - Driver payout batched daily via bank transfer

## Key Decisions to Highlight

1. Redis GEO for driver location — sub-ms geospatial queries at high write throughput
2. Fan-out to N drivers simultaneously — faster match than sequential offers
3. WebSocket presence service — real-time bidirectional push without polling
4. Geohash grid for spatial queries — O(1) lookup by cell, simple to explain
