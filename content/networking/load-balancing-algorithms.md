---
title: "Load Balancing Algorithms"
tags: [load-balancing, round-robin, consistent-hashing, least-connections, geographic-routing]
difficulty: medium
estimated_time: 20min
---

## Overview

The load balancing algorithm determines *which* backend server handles each incoming request. The right algorithm depends on the nature of your workload: are requests uniform in cost? Do clients need affinity to a specific server? Is the backend pool static or frequently changing? Choosing the wrong algorithm causes uneven load distribution, hot spots, or cache inefficiency even when you have plenty of total capacity.

**Round Robin** cycles through servers sequentially — simple and even when requests are homogeneous. **Weighted Round Robin** assigns proportional shares (useful for heterogeneous instances or canary deployments). **Least Connections** routes to the server with the fewest active connections, which handles variable-cost requests better than round robin but requires connection count tracking. **Least Response Time** combines active connections with observed latency — used in Envoy's `LEAST_REQUEST` with active health metrics.

**Consistent Hashing** maps both servers and requests onto a virtual ring using a hash function. Each request is routed to the nearest server clockwise on the ring. Adding or removing a server only remaps a fraction (`1/n`) of keys instead of rehashing everything — critical for cache servers, session stores, and stateful services. **Geographic / Latency-based routing** uses the client's location (IP geolocation or resolver location) to route to the nearest region — a DNS-level decision, not a per-request algorithm.

```
Round Robin:
  Request 1 -> Server A
  Request 2 -> Server B
  Request 3 -> Server C
  Request 4 -> Server A  (wraps around)

Weighted Round Robin (A=50%, B=30%, C=20%):
  Requests: A A A B B C A A A B B C ...

Least Connections:
  Server A: 10 active conns
  Server B:  3 active conns  <-- next request goes here
  Server C:  7 active conns

Consistent Hashing Ring:
        [Server A at hash 100]
       /                      \
  [ring 0]                [ring 300]
       \                      /
        [Server B at hash 200]

  Request hash=150 --> Server B (nearest clockwise)
  Request hash=350 --> Server A (wraps to start)

  Adding Server D at hash 250:
    Only requests in (200, 250] remapped from B to D
    All other requests unchanged

Virtual nodes (vnodes):
  Each physical server appears N times on the ring (e.g., N=150)
  Improves uniform distribution when server count is small
```

## When to Use

- **Round Robin**: Stateless services with homogeneous request cost (API servers, web servers). The default and simplest choice — use it unless you have a specific reason not to.
- **Weighted Round Robin**: Mixed-capacity backend fleet, canary deployments (5% weight to new version), or spot/on-demand instance mixing.
- **Least Connections**: Long-lived connections (WebSockets, gRPC streaming, database proxies) or highly variable request durations (ML inference, report generation). Avoids hot spots that round robin creates when some requests take 100x longer than others.
- **Consistent Hashing**: Distributed caches (Memcached, Redis Cluster), session affinity without sticky cookies, sharded services where a given key must always reach the same shard. Essential for cache hit rate — random routing degrades cache efficiency linearly with server count.
- **Geographic routing**: Multi-region systems where latency to the user dominates (Route 53 latency routing, Cloudflare Traffic Manager, Anycast). DNS-level, not per-request.
- **Random with two choices (Power of Two)**: Pick two random servers, route to the one with fewer connections. Achieves near-optimal distribution with O(1) overhead — used in large-scale proxy systems.

## Trade-offs & Gotchas

- Round robin breaks down when requests have highly variable cost — a few slow requests can saturate one server while others are idle.
- Consistent hashing requires virtual nodes (vnodes) for even distribution with small cluster sizes. Without them, physical servers cluster unevenly on the ring due to hash function variance.
- Hot keys in consistent hashing: a single extremely popular key cannot be load-balanced across servers (it always maps to one server). Solutions: key replication, dedicated shard, or client-side request sharding with a multiplier.
- Sticky sessions (IP hash, cookie-based affinity) couple clients to specific backends, making it harder to scale down or replace instances without disrupting active sessions. Prefer stateless backends.
- Least Connections requires a centralized counter or gossip-based coordination in distributed load balancers. Approximate counts (per-process, not global) are common — accurate enough in practice.
- **Rendezvous hashing (HRW)** is an alternative to consistent hashing that doesn't require a ring: for each key, compute `hash(key, server_id)` for all servers and pick the highest. Simpler but O(n) per lookup — practical for small server sets.
- Geographic routing introduces complexity: you need health checks per region and failover logic (if the nearest region is down, route to the next nearest).

## Key Points for Interviews

- Round Robin = simplest, good for homogeneous stateless workloads. Least Connections = better for variable-cost requests.
- Consistent Hashing = essential for distributed caches. Adding/removing nodes remaps only `1/n` of keys. Vnodes improve distribution.
- Hot key problem in consistent hashing: a viral key always maps to one server — mention replication or special handling.
- Geographic routing is DNS-level (Route 53 latency routing) — not per-request; TTL-bounded responsiveness.
- Power of Two random choices achieves near-perfect balancing with minimal overhead — worth mentioning for large-scale systems.
- Weighted routing enables canary deploys: send 1-5% to new version, observe error rates, then ramp up.
- For stateful services (caches, sessions), consistent hashing is the right default. For stateless services, round robin or least connections.
