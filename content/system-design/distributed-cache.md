---
title: "Design a Distributed Cache (e.g., Redis)"
tags: [caching, consistent-hashing, eviction, replication, ttl, cluster]
difficulty: medium
estimated_time: 45min
companies: [Google, Amazon, Meta, Microsoft, Stripe]
---

## Problem Statement

Design a distributed in-memory cache that can store key-value data at high
throughput and low latency, supporting horizontal scaling across many nodes.

## Clarifying Questions

Ask these before designing:

- Scale? (Assume 1M reads/sec, 100K writes/sec across the cluster)
- Data model? (Key-value; values up to 1 MB)
- Consistency requirement? (Eventual consistency acceptable; strong preferred for writes)
- Persistence needed? (Optional — cache should survive brief node restarts)
- Eviction policy? (LRU by default, configurable)
- Read/write latency target? (< 1ms p99 for reads)

## Capacity Estimates

  Memory per node: 64 GB (reasonable server)
  Total data:      1 TB → ~16 nodes at 64 GB each (with replication, ~32 nodes)
  Throughput:      1M reads/sec across 16 nodes = ~62K reads/sec per node (easy)
  Network:         1M reads * 1 KB avg = 1 GB/s total — need 10 GbE NICs

## High-Level Design

```
[Client]
   |
   v
[Cache Client Library]
(consistent hash → node selection)
   |
   ├──> [Cache Node 1]  (primary shard A, B, C)
   ├──> [Cache Node 2]  (primary shard D, E, F)
   ├──> [Cache Node 3]  (primary shard G, H, I)
   └──> [Cache Node N]  ...

Each primary node:
   |
   v
[Replica Node] (standby, handles reads in read-scale mode)

[Cluster Manager / Config Service]
(tracks node membership, handles failover)
```

## Consistent Hashing

Distributing keys across nodes: consistent hashing minimizes key remapping
when nodes are added or removed.

**How it works:**
  - Map both nodes and keys to positions on a virtual ring (hash ring)
  - A key is owned by the first node clockwise from its hash position
  - Node added: only the keys between the new node and its predecessor move
  - Node removed: only that node's keys must be redistributed

**Virtual nodes (vnodes):**
  - Each physical node maps to K virtual positions on the ring (e.g., 150 vnodes)
  - Evens out load distribution; avoids "hot zone" when nodes have different capacities
  - Prevents single-node overload when a neighbor fails

**Without consistent hashing (modulo hashing):**
  - key → node = hash(key) % num_nodes
  - Adding one node remaps nearly ALL keys → thundering herd on the DB

## Data Partitioning

With N nodes, each node owns ~1/N of the keyspace:

  Node 1: keys in ring position [0, 1/N)
  Node 2: keys in ring position [1/N, 2/N)
  ...

The client library (not the server) performs routing:
  1. Hash the key → position on ring
  2. Find the responsible node
  3. Send request directly to that node

This is the "smart client" pattern — no coordinator hop needed.

## Replication

Each shard is replicated to R replica nodes (R = 2 is common):

**Leader-follower (primary-replica):**
  - All writes go to the leader
  - Reads can go to leader (strong consistency) or replicas (eventual consistency)
  - Replica lag is typically < 10ms on same LAN

**Replication factor trade-offs:**
  - R=1: no redundancy; node failure loses all data in that shard
  - R=2: tolerate 1 failure; common for caches (data loss tolerable, fast rebuild)
  - R=3: tolerate 2 failures; used when cache is the system of record

On leader failure: cluster manager promotes a replica to leader within seconds.

## Eviction Policies

Cache must evict keys when memory is full. Common policies:

  LRU (Least Recently Used):
    - Evict the key not accessed for the longest time
    - Good general default; works well for temporal locality
    - Implementation: doubly-linked list + hash map (O(1) access and eviction)

  LFU (Least Frequently Used):
    - Evict the key accessed the fewest times
    - Better for stable "hot" data; resistant to one-off large scans
    - More complex to implement accurately

  TTL-based expiry:
    - Keys expire at a wall-clock time (set per-key at write time)
    - Lazy expiry: check TTL on read, delete if expired
    - Active expiry: background sweep samples random keys and deletes expired ones
    - Redis does both: lazy + active expiry with a configurable sampling rate

  FIFO / Random:
    - Simple but rarely optimal; rarely used in production

## Write Strategies

Three patterns for keeping the cache consistent with the DB:

  Cache-aside (lazy loading):
    - App checks cache first; on miss, loads from DB and populates cache
    - Pros: only cache what's needed; DB is always authoritative
    - Cons: cold start (empty cache after restart); thundering herd on cold miss

  Write-through:
    - App writes to cache AND DB synchronously on every write
    - Pros: cache always up to date
    - Cons: write latency doubles; cache filled with data that may not be read

  Write-behind (write-back):
    - App writes to cache only; cache asynchronously flushes to DB
    - Pros: very fast writes
    - Cons: risk of data loss if cache node crashes before flush

## Deep Dives

### Thundering Herd / Cache Stampede
  - Popular key expires → many concurrent requests all go to DB simultaneously
  - Mitigation 1: add random jitter to TTL (not all keys expire at the same second)
  - Mitigation 2: probabilistic early recomputation (refresh before expiry based on
    remaining TTL and compute time)
  - Mitigation 3: lock with a single request computing the value; others wait

### Hot Key Problem
  - A single key receives millions of reads/sec → overwhelms one node
  - Solution: replicate hot keys to multiple nodes; client randomly selects replica
  - Or: local in-process cache (L1) in the client for the top-N hottest keys

### Persistence (Optional)
  - RDB snapshots: periodic full dump to disk (used by Redis)
  - AOF (append-only file): log every write command for durability
  - For a pure cache: no persistence needed; DB is the source of truth
  - For a cache-as-database use case: AOF + RDB hybrid recommended

### Cluster Membership (Gossip Protocol)
  - Nodes exchange membership state with random peers periodically
  - Convergent: eventually all nodes agree on the cluster topology
  - Failure detection: if node X doesn't respond to N gossip rounds → suspected down
  - Used by Redis Cluster, Cassandra, Consul

## Key Decisions to Highlight

1. Consistent hashing with vnodes — minimal key remapping on topology changes
2. Smart client routing — avoids coordinator hop, reduces latency
3. LRU + TTL eviction — handles both temporal and explicit expiry
4. Leader-follower replication (R=2) — tolerate one failure without sacrificing write speed
