---
title: "Multi-Region Architecture"
tags: [multi-region, active-active, active-passive, failover, replication, rto, rpo, split-brain]
difficulty: hard
estimated_time: 30min
---

## Overview

Multi-region architecture deploys your system across two or more geographic regions, providing
resilience against regional failures and lower latency for global users. It is one of the most
complex architectural decisions — coordination overhead, replication lag, and consistency
trade-offs compound. Only reach for it when your RTO/RPO requirements or global user base
justify the cost and complexity.

Active-passive (warm/cold standby) designates one region as primary for all traffic. The secondary
region maintains a replica of data and infrastructure, ready to take over if the primary fails.
Warm standby keeps the secondary running at reduced capacity (fast failover, ~minutes). Cold
standby shuts down non-essential resources in the secondary (slower failover, lower cost). All
writes go to the primary; the secondary receives asynchronous replication updates. RTO (recovery
time) is minutes to an hour depending on automation.

Active-active runs traffic simultaneously in multiple regions. Each region handles a subset of
users (typically by geography). Write coordination is the hard problem: concurrent writes to the
same record in different regions can conflict. Solutions include: master-per-shard routing (user
always writes to their home region), last-write-wins (LWW) with vector clocks, or using a globally
consistent database (Spanner, CockroachDB) that handles distributed transactions. Active-active
achieves lower RTO and lower read latency, but significantly higher complexity.

RTO (Recovery Time Objective) is the maximum acceptable downtime after a failure. RPO (Recovery
Point Objective) is the maximum acceptable data loss (measured in time). Synchronous replication
gives RPO=0 (no data loss) but increases write latency by the inter-region network RTT (~70ms
US-EU). Asynchronous replication reduces write latency but introduces replication lag, meaning
RPO > 0 if the primary fails before the secondary catches up.

## When to Use

- **Active-passive (warm standby)**: regulated industries requiring DR, services with RTO < 1hr
  and RPO < 15min, or when global write distribution isn't needed.
- **Active-active**: global products where latency for users in different continents matters,
  or when you need RTO near zero.
- **Global databases** (DynamoDB Global Tables, Spanner, CockroachDB): when you need multi-region
  writes with automatic conflict resolution and don't want to build it yourself.

## Trade-offs & Gotchas

- Synchronous cross-region replication: US-East to EU adds ~70ms RTT to every write. For a
  database with synchronous standby, this directly increases write latency.
- Asynchronous replication lag means your secondary can be seconds to minutes behind the primary.
  If you fail over, you may lose that window of writes (RPO > 0).
- DNS-based failover is limited by TTL: even with TTL=60s, some resolvers cache longer.
  AWS Route 53 health-check failover typically takes 1–2 minutes end-to-end.
- Split-brain: both regions think they are the primary and accept writes independently, causing
  data divergence. Prevent with: fencing tokens, primary lease with heartbeat, or quorum-based
  writes that require acknowledgment from N/2+1 nodes.
- Data sovereignty / compliance: GDPR requires EU user data to stay in EU. Multi-region designs
  must account for data residency requirements — you may not be able to replicate all data
  globally.
- Cost: active-active doubles (or more) your infrastructure cost. Active-passive warm standby
  is typically 30–50% more than single-region.

## Architecture Diagram

```
  Active-Passive (Warm Standby):
  [Route 53]
  health check on primary -->
       |
  [Primary: us-east-1]       [Standby: eu-west-1]
  [All traffic]              [Warm replica]
       |                           ^
  [RDS Primary] ---async replication--> [RDS Replica]
                                        (promoted on failover)

  Active-Active:
  [Route 53 / Global Accelerator]
     |                         |
  [us-east-1]            [eu-west-1]
  [US Users]             [EU Users]
       |                       |
  [DynamoDB Global Table] -- bidirectional replication -- [DynamoDB Global Table]
  (eventual consistency, ~1s replication lag)

  RTO / RPO Matrix:
                    Cold Standby | Warm Standby | Active-Active
  RTO               Hours        | Minutes      | Seconds
  RPO               Minutes      | Seconds      | Near-zero
  Cost Overhead     Low (+20%)   | Medium (+40%)| High (+100%)
  Complexity        Low          | Medium       | High
```

## Key Interview Points

- Always clarify RTO and RPO requirements before proposing multi-region — don't over-engineer.
- Active-passive is simpler and sufficient for most DR requirements. Active-active only when
  global write latency or near-zero RTO is required.
- Synchronous replication = RPO 0, higher write latency. Async replication = lower latency,
  non-zero RPO.
- Split-brain is the most dangerous failure mode in active-active — discuss how you'd prevent it
  (quorum writes, primary leases, fencing).
- Data residency (GDPR, CCPA) may constrain which data can be replicated across regions — always
  mention this for products with EU users.
- DynamoDB Global Tables and Spanner abstract away multi-region write complexity — mention them
  as alternatives to rolling your own replication logic.
