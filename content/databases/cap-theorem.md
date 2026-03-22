---
title: "CAP Theorem"
tags: [cap-theorem, consistency, availability, partition-tolerance, distributed-systems]
difficulty: medium
estimated_time: 15min
---

## Overview

**CAP theorem** (Brewer's theorem, 2000) states that a distributed system can guarantee at most two of three properties simultaneously: **Consistency** (C), **Availability** (A), and **Partition tolerance** (P). In practice, network partitions are unavoidable in any distributed system — so the real choice is between C and A when a partition occurs.

**Consistency** in CAP means **linearizability**: every read receives the most recent write or an error. Every node in the cluster sees the same data at the same time. This is stronger than ACID consistency (which is about constraint validity). **Availability** means every request receives a non-error response — though it may not be the most recent data. The system never returns an error or times out. **Partition tolerance** means the system continues operating even if network partitions split nodes into groups that cannot communicate.

When a partition happens, a **CP system** sacrifices availability to maintain consistency: it refuses to serve reads/writes on nodes that cannot confirm they have the latest data (returns error or times out). Examples: HBase, Zookeeper, etcd, Redis Sentinel in strict mode. A **CA system** would require no partitions — impossible in a distributed system, so pure CA systems are only possible on a single machine (traditional RDBMS on one node). An **AP system** sacrifices consistency to maintain availability: nodes serve reads and writes even when partitioned, accepting that different nodes may return different values temporarily. Examples: Cassandra, DynamoDB, CouchDB. Data eventually converges when the partition heals.

## When to Use

- **CP**: financial transactions, configuration stores, distributed locks, anything where serving stale data is worse than refusing to serve at all. Use etcd, ZooKeeper, or CockroachDB.
- **AP**: user profiles, shopping carts, social feeds, DNS — where availability is more important than perfect consistency and stale reads are acceptable. Cassandra and DynamoDB are designed AP.
- **Single-node RDBMS**: technically CA because partitions aren't a concern, but this doesn't scale horizontally.

## Trade-offs & Gotchas

- **CAP is a spectrum, not a binary**: most real systems tune the CP/AP trade-off dynamically. Cassandra's tunable consistency (ONE, QUORUM, ALL) lets you shift toward CP (use QUORUM or ALL) or AP (use ONE) per operation.
- **"Consistency" in CAP != "Consistency" in ACID**: CAP consistency = linearizability (single-copy semantics). ACID consistency = constraint validity. These are different concepts with the same name — a common source of confusion in interviews.
- **Availability in CAP is absolute**: a system that returns stale data is "available" in CAP terms. A system that returns a timeout is not. This is a weaker definition than SLA-based availability (99.9% uptime).
- **CAP doesn't cover latency**: a CP system can be "available" (not returning errors) while having 10-second response times during a partition. PACELC extends CAP to address this.
- **Partition tolerance isn't optional**: network partitions happen in all distributed systems — packet loss, NIC failures, rack switches, datacenter link drops. Any system claiming CA is either single-node or misusing the term.
- **BASE vs ACID**: AP systems are often described as **BASE** — **B**asically **A**vailable, **S**oft state, **E**ventually consistent. This is the philosophical counterpart to ACID for distributed NoSQL systems.

## Architecture Diagram

```
CAP Triangle:
              Consistency (C)
                   /\
                  /  \
                 / CP \
                /      \
               /--------\
              / CA  | AP \
             /      |     \
        Availability(A) -- Partition Tolerance(P)

  CA: Traditional RDBMS (single node) - no partitions to tolerate
  CP: HBase, ZooKeeper, etcd, Redis (strict) - refuse if uncertain
  AP: Cassandra, DynamoDB, CouchDB - serve stale, reconcile later

Network Partition Scenario:
  [Node A] ===PARTITION=== [Node B]
  Client writes to A: x = 5
  Client reads from B: x = ???

  CP choice: B returns ERROR (consistent, not available)
  AP choice: B returns x = 4 (stale, available)
  After partition heals: AP systems reconcile x -> 5

Cassandra Tunable Consistency:
  Write consistency ONE   -> fast, AP behavior (1 node confirms)
  Write consistency QUORUM-> balanced, partial CP (majority confirms)
  Write consistency ALL   -> slow, CP behavior (all nodes confirm)
```

## Key Points for Interviews

- The real CAP choice is C vs A during a partition — P is not optional for any distributed system.
- CAP Consistency (linearizability) is NOT the same as ACID Consistency (constraints). Clarify which you mean.
- Most AP systems offer tunable consistency — you can dial toward CP per-request (Cassandra QUORUM, DynamoDB strongly consistent reads).
- Know that CAP ignores latency — introduce PACELC as the more practical extension when the interviewer is ready to go deeper.
- BASE (Basically Available, Soft state, Eventually consistent) is the AP counterpart to ACID.
- Real-world examples: ZooKeeper/etcd = CP (used for leader election precisely because of this). Cassandra/DynamoDB = AP (used for user data because availability matters more).
- Don't just recite the triangle — say what the trade-off means in operational terms: "during a partition, a CP system's nodes will refuse requests rather than risk serving stale data."
