---
title: "Transactions & ACID"
tags: [transactions, acid, atomicity, consistency, isolation, durability, mvcc, isolation-levels]
difficulty: medium
estimated_time: 20min
---

## Overview

A **transaction** is a sequence of database operations treated as a single unit of work. Either all operations succeed (**commit**) or none of them do (**rollback**). Transactions are the mechanism that turns a database into a reliable system rather than a fast file. **ACID** is the set of properties that define a correct transaction implementation.

**Atomicity**: all-or-nothing. A transaction either commits completely or is rolled back completely — no partial writes survive. If power fails mid-transaction, the database uses its write-ahead log (WAL) to undo any partially-applied changes on restart. **Consistency**: a transaction takes the database from one valid state to another, respecting all defined constraints (foreign keys, unique constraints, check constraints). Consistency is partly a database guarantee and partly an application contract — the app must write logically valid data. **Durability**: once a transaction commits, it is permanent. The WAL ensures committed data survives crashes; the log is flushed to disk before the `COMMIT` acknowledgement is sent. **Isolation**: concurrent transactions behave as if they run serially — one cannot see another's intermediate state. Isolation is the most nuanced property and is controlled by **isolation levels**.

**Isolation levels** trade correctness for performance. Four standard levels (weakest to strongest): **Read Uncommitted** — can read uncommitted (dirty) writes from other transactions; almost never used. **Read Committed** — only sees committed data; prevents dirty reads but allows non-repeatable reads (same row read twice in a transaction may return different values). The default in PostgreSQL and Oracle. **Repeatable Read** — a transaction sees a consistent snapshot of committed data as of its start time; prevents non-repeatable reads but allows phantom reads (new rows matching a query may appear). MySQL InnoDB default. **Serializable** — full isolation; transactions appear to execute one at a time; prevents all anomalies but is the most expensive. Most databases implement isolation via **MVCC** (Multi-Version Concurrency Control): writes create new row versions instead of overwriting, and each transaction reads a consistent snapshot without blocking readers.

## When to Use

- **Read Committed**: the safe default for most OLTP workloads. Prevents dirty reads with minimal overhead.
- **Repeatable Read**: financial calculations, inventory checks — anywhere the same row is read twice in a transaction and must return consistent results.
- **Serializable**: counter increments, seat/ticket reservations, any check-then-act pattern where concurrent updates would produce incorrect results (e.g., "book the last seat"). Accept the throughput cost.
- **Explicit transactions** (`BEGIN` / `COMMIT`): whenever you perform multiple related writes that must all succeed together — transferring money between accounts, creating an order with its line items.

## Trade-offs & Gotchas

- **Lost update**: two transactions read the same row, both compute a new value based on what they read, and both write — one overwrites the other's update. Classic example: two sessions both read a counter as 5, both add 1, both write 6 instead of 7. Fix: `SELECT ... FOR UPDATE` (pessimistic lock), `UPDATE ... WHERE value = 5` (optimistic lock / CAS), or `UPDATE counter SET value = value + 1` (atomic increment in SQL).
- **Phantom reads**: a transaction reads all rows matching a condition, another transaction inserts a new matching row and commits, the first re-queries and sees the phantom row. Prevented by Serializable; Repeatable Read prevents it in PostgreSQL via MVCC snapshots but not in all databases.
- **Deadlock**: two transactions each hold a lock the other needs. Databases detect and resolve deadlocks by rolling back one transaction. Application code must handle `deadlock detected` errors and retry. Prevent deadlocks by always acquiring locks in a consistent order.
- **Long transactions are dangerous**: they hold locks, prevent MVCC garbage collection (table bloat in Postgres), and increase rollback cost. Keep transactions short. Never hold a transaction open while waiting for user input.
- **`COMMIT` latency = fsync latency**: durability requires flushing the WAL to disk. Cloud databases on network storage can have fsync latency of 1–10ms per commit. Batch multiple writes into one transaction to amortize this cost.
- **Distributed transactions (2PC)**: spanning a transaction across multiple databases or services requires two-phase commit — coordinator asks all participants to prepare, then commits if all agree. 2PC is slow and the coordinator is a single point of failure. Prefer sagas (compensating transactions) for microservices.

## Architecture Diagram

```
ACID Properties:
  A - Atomicity:    [op1][op2][op3] -> all commit, or all rollback
  C - Consistency:  constraints hold before and after transaction
  I - Isolation:    concurrent txns don't see each other's partial state
  D - Durability:   committed data survives crashes (WAL flushed to disk)

Isolation Levels (weakest -> strongest):
  Level             | Dirty Read | Non-Repeatable | Phantom Read
  ------------------+------------+----------------+-------------
  Read Uncommitted  |    YES     |      YES       |     YES
  Read Committed    |    no      |      YES       |     YES    <- PG default
  Repeatable Read   |    no      |      no        |     YES    <- MySQL default
  Serializable      |    no      |      no        |     no

MVCC (Multi-Version Concurrency Control):
  Write txn (T2) updates row:    row_v1(xmin=T1) | row_v2(xmin=T2, xmax=inf)
  Read txn (T1, snapshot @ T1):  sees row_v1 only (T2 not in snapshot)
  -> No read-write blocking; readers never block writers

Lost Update - Problem:
  T1: READ x=5   T2: READ x=5
  T1: WRITE x=6  T2: WRITE x=6  <- T1's update lost!

Lost Update - Fix (SELECT FOR UPDATE):
  T1: SELECT x=5 FOR UPDATE  <- acquires lock
  T2: SELECT x=5 FOR UPDATE  <- blocks, waits
  T1: WRITE x=6, COMMIT      <- lock released
  T2: wakes, reads x=6, WRITE x=7, COMMIT  <- correct
```

## Key Points for Interviews

- Know all four ACID properties and give a concrete example of each.
- Isolation levels are the most nuanced — be able to describe what anomaly each level prevents and which it allows.
- Lost update is the most common ACID pitfall in application code. Know `SELECT FOR UPDATE`, optimistic locking (CAS), and atomic SQL increments as fixes.
- MVCC is how modern databases achieve isolation without blocking reads — mention it to show depth.
- Long transactions are a production anti-pattern — table bloat, lock contention, slow rollbacks.
- For distributed systems, 2PC is slow and fragile. Mention the saga pattern as the modern alternative for cross-service transactions.
- Serializable prevents all anomalies but has the highest cost. Most apps don't need it — audit your actual anomaly risk and choose the weakest level that covers it.
