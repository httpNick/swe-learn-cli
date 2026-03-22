---
title: "Database Sharding"
tags: [sharding, horizontal-scaling, consistent-hashing, shard-key, partitioning, hotspot]
difficulty: hard
estimated_time: 25min
---

## Overview

**Sharding** (horizontal partitioning) splits a large dataset across multiple database nodes, each owning a subset of the data called a **shard**. Unlike replication (which copies the same data), sharding distributes different data to different nodes — each shard holds data no other shard holds. The goal is to scale both storage and write throughput beyond what a single machine can handle.

**Vertical scaling** (scaling up) adds more CPU, RAM, or faster disks to a single machine. It is simpler and should be exhausted first — a modern cloud instance with 96 vCPUs, 384 GB RAM, and NVMe SSDs handles enormous workloads. Vertical scaling has a hard ceiling (no single machine beyond a certain size exists) and creates a single point of failure. **Horizontal scaling** (scaling out) adds more nodes, each handling a portion of the load. More complex to operate but theoretically unbounded.

**Shard key selection** is the most consequential sharding decision. A bad shard key creates **hotspots** — one shard receiving disproportionate traffic while others sit idle. Hotspots are the #1 sharding failure mode. An ideal shard key distributes writes evenly, maps naturally to the most common access pattern, and does not change after being set. Common approaches: hash of user ID (even distribution, no range queries), range partitioning by timestamp (enables time-range scans but risks write hotspots on the most-recent shard), and compound keys.

**Consistent hashing** is the standard algorithm for assigning keys to shards. Keys and shard nodes are placed on a virtual ring using their hash values. A key is assigned to the first node clockwise from its position on the ring. When a node is added or removed, only the keys between the new node and its predecessor need to be remapped — O(K/N) keys move rather than O(K). **Virtual nodes** (vnodes) assign each physical node multiple positions on the ring, improving load balance when nodes have different capacities and reducing hot-spot risk during node failures.

## When to Use

- **Don't shard until you must**: operational complexity is high. Try vertical scaling, read replicas, caching, and query optimization first.
- **Shard by user ID**: default for multi-tenant SaaS — all data for a user lives on one shard, enabling efficient single-user queries.
- **Shard by geographic region**: data residency requirements, or latency — route EU users to EU shards.
- **Range sharding by time**: time-series / event data where you query by time range. Archive old shards as cold storage.
- **Hash sharding**: when your primary access pattern is key-value lookups and you need the most even write distribution.

## Trade-offs & Gotchas

- **Cross-shard queries are expensive**: a `JOIN` or aggregation spanning all shards requires a scatter-gather — query every shard, then merge results in the application layer. This is slow and should be avoided. Design your shard key so that the most common queries touch a single shard.
- **Hotspots from bad shard keys**: sharding by `last_name` puts all "Smith" users on one shard; sharding by `created_at` puts all new writes on the latest shard. Test your key's distribution before committing.
- **Resharding is painful**: adding or removing shards requires moving data. Without consistent hashing, this means remapping all keys. Even with consistent hashing, online resharding requires care — dual-read or background migration strategies.
- **Transactions across shards**: ACID transactions do not span shards natively. Cross-shard transactions require distributed two-phase commit (2PC), which is slow and complex. Design to avoid them; use saga pattern or eventual consistency where unavoidable.
- **Shard key immutability**: if a row's shard key changes (e.g., a user changes their username and it's the shard key), you must move the row to a new shard — complex and dangerous. Choose shard keys that never change (internal IDs, UUIDs).
- **Uneven shard growth**: range shards can grow at different rates over time — the "active" shard always has more recent (and possibly hotter) data. Plan for periodic rebalancing.

## Architecture Diagram

```
Horizontal vs Vertical Scaling:
  Vertical (Scale Up):          Horizontal (Scale Out / Sharding):
  +------------------+          +--------+  +--------+  +--------+
  | Single BIG node  |          | Shard1 |  | Shard2 |  | Shard3 |
  | CPU: 96 cores    |          | u:0-33%|  | u:34-66|  |u:67-100|
  | RAM: 384 GB      |          +--------+  +--------+  +--------+
  | Limit: 1 machine |          | Limit: add more shards |
  +------------------+          +------------------------+

Consistent Hashing Ring (N=4 nodes):
           0
         / | \
  Node D   |   Node A
   270     |     90
         \ | /
  Node C   |   Node B
          180

  Key "user:1" hashes to position 40  --> Node A (nearest clockwise)
  Key "user:2" hashes to position 200 --> Node C
  Add Node E at position 60: only keys 40-60 move from A to E

Shard Key Comparison:
  user_id (hash)   -> even distribution, no range scans
  email (hash)     -> even distribution, lookups by email only
  created_at       -> range queries OK, new-record hotspot
  region           -> geo-local, uneven if regions differ in size
  last_name        -> bad: alphabetic clustering creates skew
```

## Key Points for Interviews

- Sharding is a last resort. State what you'd try first: vertical scaling, read replicas, caching, indexing.
- Lead with shard key selection and justify it. Interviewers want to hear you anticipate hotspots.
- Consistent hashing minimizes key movement on topology changes — describe the ring and virtual nodes.
- Cross-shard operations are the Achilles heel: scatter-gather reads, no native ACID transactions. Design your access patterns to avoid them.
- Mention virtual nodes (vnodes) for load balancing across heterogeneous hardware.
- Resharding requires a migration strategy — dual-read, background copy, or blue-green shard promotion.
- In practice, managed services (DynamoDB, Spanner, CockroachDB, Vitess for MySQL) handle sharding transparently. Mentioning them shows production awareness.
