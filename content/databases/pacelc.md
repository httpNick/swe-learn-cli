---
title: "PACELC Theorem"
tags: [pacelc, cap-theorem, latency, consistency, availability, partition, trade-offs]
difficulty: hard
estimated_time: 15min
---

## Overview

**PACELC** (Daniel Abadi, 2012) extends CAP theorem by addressing its most significant omission: **latency**. CAP only characterizes system behavior during a network partition. PACELC asks: what about normal operation when there is no partition? The theorem states: if a Partition occurs (P), the system must trade off between Availability (A) and Consistency (C) — just as CAP says. Else (E), when the system is running normally, the system must trade off between Latency (L) and Consistency (C).

The latency-consistency trade-off is always present, even without failures. Achieving **strong consistency** (linearizability) in a distributed system requires coordination — waiting for acknowledgements from multiple nodes before confirming a write. That coordination takes time: network round trips, quorum acknowledgements, consensus protocol messages. **Lower latency** means reducing that coordination, which means accepting weaker consistency guarantees (a write may be acknowledged before all replicas confirm it, so reads may briefly see stale data). This is the E/L trade-off that CAP completely ignores.

PACELC classification uses the format **PA/EL** or **PC/EC** etc. **PA/EL systems** (Partition Availability, Else Latency): prioritize availability during partitions and low latency during normal operation. Examples: Cassandra, DynamoDB, Riak. Writes are acknowledged quickly from a small number of replicas; consistency is eventually achieved. **PC/EC systems** (Partition Consistency, Else Consistency): prioritize consistency in both modes. Examples: HBase, Spanner, VoltDB. Writes require coordination and may be slower but data is always consistent. **PC/EL** and **PA/EC** represent mixed strategies — some systems are configurable (Cassandra with tunable consistency can behave as PA/EL or PC/EC depending on the quorum setting chosen).

## When to Use

- **PA/EL (Cassandra, DynamoDB defaults)**: user activity feeds, shopping carts, messaging, any workload where a few milliseconds of write latency matter and brief inconsistency is tolerable.
- **PC/EC (Spanner, VoltDB, etcd)**: financial ledgers, distributed locking, configuration management, inventory systems where correctness is paramount and latency is a secondary concern.
- **PC/EL (MySQL Cluster, some DynamoDB configs)**: systems that need partition safety but can trade latency for consistency in normal operation — rare in practice.
- Use PACELC vocabulary in interviews to signal that you know CAP is incomplete and that you think about latency in the normal (non-partitioned) case.

## Trade-offs & Gotchas

- **CAP is a subset**: PACELC subsumes CAP. Every system has a PACELC classification; CAP only describes half of it (the P branch). When an interviewer asks about CAP, you can score points by noting the E branch.
- **Tunable systems complicate classification**: Cassandra with `QUORUM` consistency behaves as PC/EC; with `ONE` it behaves as PA/EL. The PACELC label applies to the default/typical configuration, not every possible setting.
- **Latency vs consistency is always present**: even on a single datacenter with no partitions, achieving linearizable reads requires checking that your read reflects the latest committed write — which may mean a round trip to confirm with a replica. Skip that, and you get lower latency but potentially stale reads.
- **Geo-distribution amplifies the E trade-off**: if replicas span continents, achieving EC (Else Consistency) requires waiting for a write to be acknowledged in multiple regions — adding 50–200ms of latency per write. Spanner uses GPS-synchronized clocks (TrueTime) to minimize this overhead.
- **Eventual consistency is not chaos**: in PA/EL systems, "eventually consistent" means replicas converge within milliseconds to seconds under normal conditions — not arbitrary delay. The window of inconsistency is bounded and predictable.
- **Write path vs read path**: PACELC trade-offs often apply differently to reads and writes. A system might acknowledge writes fast (EL) but require a quorum read to guarantee freshness (EC for reads). Read your database's documentation carefully.

## Architecture Diagram

```
PACELC Decision Tree:
                      System
                        |
              +---------+---------+
              |                   |
         Partition?            No Partition
              |                   |
    +---------+--------+   +------+------+
    |                  |   |             |
  Availability    Consistency  Latency  Consistency
  (PA)            (PC)         (EL)     (EC)

PACELC Classification of Common Systems:
  System        | P choice | E choice | Notes
  --------------+----------+----------+---------------------------
  Cassandra     | A (PA)   | L (EL)   | default; tunable to PC/EC
  DynamoDB      | A (PA)   | L (EL)   | default; strongly consistent reads = PC/EC
  Riak          | A (PA)   | L (EL)   | Dynamo-style
  HBase         | C (PC)   | C (EC)   | HDFS-backed, strong consistency
  Spanner       | C (PC)   | C (EC)   | TrueTime, external consistency
  VoltDB        | C (PC)   | C (EC)   | in-memory, ACID
  MySQL (InnoDB)| C (PC)   | C (EC)   | single-leader, sync replication
  etcd/ZooKeeper| C (PC)   | C (EC)   | Raft/Zab consensus

Latency Cost of Consistency (E branch):
  EL (low latency): write -> 1 node ACK -> return   ~1ms
  EC (consistency): write -> quorum ACK -> return   ~5-20ms
  EC geo-dist:      write -> multi-DC ACK -> return ~50-200ms
```

## Key Points for Interviews

- Lead with PACELC when discussing distributed database trade-offs — it's more precise than CAP alone.
- The E/L trade-off is always present, even in normal operation. Consistency costs latency.
- Know the PACELC classification of at least four systems: Cassandra (PA/EL), Spanner (PC/EC), DynamoDB (PA/EL default), etcd (PC/EC).
- Tunable consistency (Cassandra) means the PACELC classification shifts per operation — a powerful design point.
- Google Spanner's TrueTime API is the most famous attempt to reduce EC latency by using GPS/atomic clocks to bound clock uncertainty, enabling external consistency with minimal coordination overhead.
- In system design interviews, after describing CAP, add: "PACELC extends this — even without partitions, strong consistency requires coordination that adds latency. That's why Cassandra defaults to eventual consistency: it prioritizes low write latency in normal operation."
