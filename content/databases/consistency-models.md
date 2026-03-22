---
title: "Consistency Models"
tags: [consistency, eventual-consistency, strong-consistency, linearizability, causal-consistency, read-your-writes, monotonic-reads]
difficulty: hard
estimated_time: 20min
---

## Overview

A **consistency model** is a contract between a distributed system and its clients, defining what guarantees a client can expect when reading data that may have been written by another client on a different node. Consistency models form a hierarchy — stronger models provide more guarantees but cost more in latency and availability. Weaker models allow replicas to diverge temporarily in exchange for speed.

**Strong consistency** (linearizability) is the gold standard. Every operation appears to take effect instantaneously at a single point in time, and every client sees the same total order of operations. After a write completes, any subsequent read from any node returns that value. This is what most developers intuitively expect but is expensive to implement — it requires coordination (quorum acknowledgements, consensus protocols) that adds latency. Used by: etcd, ZooKeeper, Spanner, CockroachDB, and SQL databases with synchronous replication.

**Causal consistency** is weaker than linearizability but stronger than eventual consistency. If operation A causally precedes operation B (A happened before B, or A's result influenced B), then any process that sees B must also see A. Operations with no causal relationship may be observed in any order. Example: if Alice posts a message and Bob replies to it, anyone who sees Bob's reply must also see Alice's post. Causal consistency can be implemented without global coordination, making it much faster than linearizability while still preserving meaningful ordering guarantees. Used by: MongoDB causal sessions, some Dynamo configurations.

**Eventual consistency** is the weakest useful model: if no new writes are made to a key, all replicas will eventually converge to the same value. No timing guarantee is given — convergence could happen in milliseconds or minutes. Reads may return stale data. Writes on different replicas may conflict and must be resolved (LWW, CRDTs, application merge). This is the default for AP systems (Cassandra with ONE, DynamoDB eventually consistent reads). Two practical session guarantees that sit between eventual and causal: **Read-your-writes** — after a client writes a value, that same client always reads its own write (even if other clients may not). Prevents the jarring experience of posting a comment and then not seeing it. **Monotonic reads** — a client never observes data going backward in time; if a client reads version 5, it will never subsequently read version 4 from a more lagged replica.

## When to Use

- **Linearizability**: distributed locks, leader election, financial transfers, inventory decrement (prevent overselling). ZooKeeper/etcd for coordination; Spanner/CockroachDB for globally consistent SQL.
- **Causal consistency**: social feeds (replies must appear after the post they reply to), collaborative editing, comment threads. MongoDB causal sessions are a practical implementation.
- **Read-your-writes**: any user-facing write — profile updates, post creation, preference changes. Users must see their own actions immediately or trust is broken. Implement by routing a user's reads to the leader (or a replica with a sequence number >= their last write).
- **Monotonic reads**: preventing time-travel reads in a multi-replica setup. Implement via session affinity to a single replica, or by tagging reads with a minimum replication sequence number.
- **Eventual consistency**: counters, analytics, recommendation data, shopping carts (Dynamo paper), any data where correctness of individual values matters less than availability and throughput.

## Trade-offs & Gotchas

- **Consistency is not binary**: there is a rich spectrum from eventual to linearizable. Picking the right model for each data type in your system is a design skill, not just trivia.
- **Eventual consistency surprises**: a user updates their profile photo, then refreshes and sees the old photo. This is "working as designed" in an AP system — but users find it confusing. Read-your-writes solves this specific case.
- **Stale reads under eventual consistency**: with Cassandra ONE read consistency, a read may hit a replica that hasn't received the latest write yet. The window is typically milliseconds but can grow during node recovery.
- **CRDTs (Conflict-free Replicated Data Types)** are a principled way to achieve eventual consistency without explicit conflict resolution: counters (G-Counter, PN-Counter), sets (G-Set, 2P-Set), registers (LWW-Register). The data structure is designed so that concurrent updates always merge without conflict. Used in Redis, Riak, collaborative document editors.
- **Monotonic writes vs monotonic reads**: these are different guarantees. Monotonic writes means a client's writes are applied in order (if you write v1 then v2, no replica applies v2 before v1). Monotonic reads means reads never go backward in time.
- **Read-your-writes is hard across devices**: if a user writes on their phone and immediately reads on their desktop (different sessions), standard read-your-writes implementations (session affinity) don't help. You need a version-token approach: write returns a token; all subsequent reads include that token and are satisfied only by up-to-date replicas.

## Architecture Diagram

```
Consistency Model Hierarchy (stronger -> weaker):
  Linearizability (Strong Consistency)
      |  - global total order of all ops
      |  - any read returns latest write
      v
  Sequential Consistency
      |  - all nodes see same op order
      |  - not necessarily real-time
      v
  Causal Consistency
      |  - causally related ops ordered
      |  - concurrent ops may be unordered
      v
  Read-Your-Writes  /  Monotonic Reads  (session guarantees)
      |  - weaker than causal, scoped to single client
      v
  Eventual Consistency
      - replicas converge "eventually"
      - no ordering guarantee at all

Read-Your-Writes Implementation:
  Option 1: Route user reads to leader after any write
  Option 2: Replica must have replication_lag < threshold
  Option 3: Write returns sequence number S;
            read blocked until replica reaches S

Causal Consistency Example:
  Alice: POST  "Hello"       (event A)
  Bob:   REPLY "Hi Alice"    (event B, caused by A)

  Valid:   Reader sees [A, B]  (correct causal order)
  Valid:   Reader sees [A]     (B not yet propagated)
  INVALID: Reader sees [B]     (reply without the post)

CRDT Counter (PN-Counter):
  Node1: {inc: 3, dec: 1}
  Node2: {inc: 5, dec: 2}
  Merge: max(inc) per node, max(dec) per node -> value = (3+5) - (1+2) = 5
  No conflict, commutative, associative merge
```

## Key Points for Interviews

- Know the hierarchy: linearizability > causal > monotonic reads/read-your-writes > eventual consistency.
- Read-your-writes is the minimum viable consistency for user-facing writes. Explain how to implement it (route to leader, or replication watermark tokens).
- Causal consistency is often the sweet spot: stronger than eventual, achievable without global coordination.
- Name CRDTs as the principled solution to conflict-free eventual consistency. Give an example (PN-Counter, G-Set).
- "Eventual consistency" is not a single thing — stale reads in a Cassandra cluster with ONE may resolve in 1ms; in a geo-distributed system during a partition it may be minutes. Qualify the bound.
- Distinguish between session guarantees (per-client) and system-wide guarantees. Most AP systems provide session-level read-your-writes without guaranteeing it system-wide.
