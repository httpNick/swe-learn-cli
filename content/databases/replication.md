---
title: "Database Replication"
tags: [replication, leader-follower, multi-leader, leaderless, quorum, consistency, high-availability]
difficulty: medium
estimated_time: 20min
---

## Overview

**Replication** keeps copies of the same data on multiple nodes. Goals: high availability (survive node failures), read scalability (serve reads from replicas), and geographic distribution (serve users from nearby nodes). Every replication strategy forces a trade-off between consistency, availability, and write throughput.

**Leader-follower** (primary-replica, master-slave) is the most common model. All writes go to a single leader, which writes to its log (WAL in Postgres, binlog in MySQL). Followers replay the log asynchronously or synchronously. **Synchronous replication** guarantees the follower is up-to-date before the write is acknowledged — zero data loss but higher write latency. **Asynchronous replication** acknowledges writes immediately and replicates in the background — low latency but a follower lag window where a leader crash causes data loss. Most systems use semi-synchronous: one synchronous follower, the rest async. Reads can be served from followers, but clients may see stale data (replication lag).

**Multi-leader** (active-active, multi-master) allows writes on multiple nodes simultaneously. Each leader replicates its writes to peers. Benefit: write availability even if a datacenter is down; lower write latency for geographically distributed users. Cost: **write conflicts** — two leaders can concurrently update the same row, and the system must resolve the conflict (last-write-wins, application-defined merge, or CRDT). Multi-leader is complex to reason about and is generally avoided unless geographic write locality is a hard requirement (e.g., CockroachDB, Google Spanner).

**Leaderless replication** (Dynamo-style, used by Cassandra and DynamoDB) has no designated leader. Any node accepts writes. Reads and writes are sent to multiple nodes simultaneously using **quorum** reads and writes. With N replicas, a write quorum W and read quorum R, data is consistent when W + R > N. The typical setup is N=3, W=2, R=2 — meaning reads and writes each contact a majority. If a node is unavailable, the system still makes progress as long as quorum is met. Anti-entropy processes (Merkle trees, read repair, hinted handoff) reconcile diverged replicas in the background.

## When to Use

- **Leader-follower (sync)**: financial systems, anything where data loss is unacceptable. Accept higher write latency. PostgreSQL synchronous_commit = on.
- **Leader-follower (async)**: read-heavy apps (blogs, social feeds) where replica lag is acceptable. Route analytics and reporting to replicas to offload the leader.
- **Multi-leader**: multi-datacenter writes where users in different regions must be able to write independently. Requires a conflict resolution strategy — plan it explicitly.
- **Leaderless (Cassandra/DynamoDB)**: always-on write availability, massive write throughput, global distribution, eventual consistency is acceptable. IoT ingestion, activity tracking, shopping carts.

## Trade-offs & Gotchas

- **Replication lag**: async followers may be seconds or minutes behind the leader. A user who just wrote their profile may immediately read from a replica and see the old data. Mitigate with **read-your-writes consistency**: route writes and subsequent reads for the same session to the leader (or use synchronous replication).
- **Failover complexity**: promoting a follower to leader is not trivial. The follower may be behind — promoting it means losing unreplicated writes. Automated failover tools (Patroni for Postgres, Orchestrator for MySQL) handle this but require careful configuration.
- **Split-brain**: if the leader fails and two nodes each believe they are the new leader, you get conflicting writes on both. Fencing (STONITH, distributed locks) and consensus protocols (Raft, Paxos) prevent this.
- **Quorum tuning**: setting W=1, R=1 (N=3) maximizes availability but gives no consistency — any single replica's stale value satisfies a read. W=3, R=1 maximizes read speed but writes must succeed on all replicas. Tune W+R > N for consistency.
- **Monotonic reads**: with multiple replicas, a user reading twice may see newer data first then older data if the second read hits a more lagged replica. Fix: pin a user's reads to a consistent replica (session affinity).
- **Conflict resolution in multi-leader**: Last-Write-Wins (LWW) using timestamps is lossy — NTP clock drift means "last" is unreliable. Prefer vector clocks or application-level merge logic for correctness.

## Architecture Diagram

```
Leader-Follower (Async):
  [Writes] --> [Leader]
                  |
          WAL/binlog stream
         /         |        \
  [Follower1] [Follower2] [Follower3]
  (may lag)   (may lag)   (may lag)
  [Reads can be served from any follower]

Leader-Follower (Semi-Sync):
  [Write] --> [Leader] --> [Sync Follower] --> ACK
                    \
                     --> [Async Follower1,2] (background)

Multi-Leader:
  [DC-West]              [DC-East]
  [Leader A] <-replicate-> [Leader B]
  |                          |
  Writes                   Writes
  CONFLICT if same row updated concurrently
  -> resolve: LWW | merge | CRDT

Leaderless (N=3, W=2, R=2):
  [Client]
    |-- write --> [Node1] ACK
    |-- write --> [Node2] ACK  <- 2 ACKs = quorum met
    |-- write --> [Node3] (may be down, ok)

  [Client]
    |-- read --> [Node1] v=5
    |-- read --> [Node2] v=5  <- 2 reads = quorum, return v=5
    (Node3 has v=4 but not queried enough to matter)
```

## Key Points for Interviews

- Leader-follower async is the default for most production systems — know its failure modes (lag, split-brain, failover data loss).
- Always mention replication lag when discussing replica reads. Propose read-your-writes or sticky sessions as mitigations.
- Quorum formula: W + R > N guarantees reading at least one node that has the latest write. Know this and be able to derive the trade-offs for different W/R combinations.
- Multi-leader = write conflicts. State your conflict resolution strategy upfront.
- Leaderless = Dynamo-style = Cassandra/DynamoDB. Great for write availability, weak on consistency without careful quorum tuning.
- Mention Raft or Paxos if asked about consensus — these are used to elect leaders and prevent split-brain, not for all replication.
