---
title: "Managed Cloud Databases"
tags: [rds, dynamodb, data-warehouse, elasticache, aurora, managed-databases]
difficulty: medium
estimated_time: 20min
---

## Overview

Cloud providers offer fully managed database services that eliminate the operational burden of
provisioning, patching, backups, and replication. The tradeoff is less control (can't tune the OS
or storage engine directly) and higher cost per GB than self-managed. For most system design
scenarios, managed databases are the right default choice.

Managed relational databases (RDS PostgreSQL/MySQL, Cloud SQL, Azure Database) handle automated
backups, point-in-time recovery, Multi-AZ failover, and read replicas out of the box. Multi-AZ
maintains a synchronous standby in another AZ — failover is automatic and takes 60–120 seconds.
Read replicas use asynchronous replication and can lag behind the primary by seconds to minutes;
never use them for reads that require up-to-the-second consistency.

Managed NoSQL services (DynamoDB, Cosmos DB, Firestore) trade schema flexibility and SQL
expressiveness for single-digit millisecond latency at any scale with no capacity planning for
DynamoDB on-demand mode. DynamoDB's key design constraint: you must model access patterns
upfront — ad-hoc queries that don't align with your partition key require expensive scans.

Data warehouses (Redshift, BigQuery, Snowflake) are OLAP systems: columnar storage, optimized for
analytical queries over billions of rows, not for transactional workloads. They are append-heavy,
batch-loaded, and separate from your operational database.

## When to Use

- **RDS/Aurora**: transactional workloads, relational data, existing SQL skills. Aurora is
  MySQL/Postgres-compatible with 5x throughput and auto-storage scaling.
- **DynamoDB**: key-value or simple query patterns needing massive scale and predictable latency
  (leaderboards, session stores, shopping carts, user profiles).
- **Redshift/BigQuery**: analytics, reporting, BI dashboards — queries over the full dataset.
- **ElastiCache (Redis/Memcached)**: caching layer in front of any database; also used for rate
  limiting, session storage, pub/sub.
- **Aurora Serverless**: dev/staging environments or workloads with highly variable or
  unpredictable traffic.

## Trade-offs & Gotchas

- RDS Multi-AZ failover takes 60–120s — not zero downtime. Use connection retry logic in app code.
- Read replica replication lag means reads can be stale — don't route consistency-sensitive reads
  to replicas.
- DynamoDB hot partition problem: if all traffic hits the same partition key (e.g., today's date),
  you'll throttle. Design partition keys for uniform distribution.
- DynamoDB on-demand mode is ~6x more expensive per request than provisioned with auto-scaling
  at predictable load — use provisioned for steady traffic.
- RDS connection limits are tied to instance size — at high concurrency, use RDS Proxy to pool
  connections and avoid exhausting the DB.
- Data warehouse queries can be expensive: always partition tables and use columnar formats.
  BigQuery charges by bytes scanned.

## Architecture Diagram

```
  Transactional (OLTP):
  [App] --> [RDS Proxy] --> [RDS Primary (Multi-AZ)]
                                  |
                         [RDS Read Replica] <-- read-heavy queries
                                  |
                         [RDS Standby AZ-2] <-- auto-failover

  High-Scale NoSQL:
  [App] --> [DynamoDB Table]
             partition: user_id
             sort key:  timestamp
             GSI: email -> user_id  (for login lookup)

  Analytics Pipeline:
  [RDS] --> [DMS / ETL]  --> [S3 Data Lake]
  [App events]                     |
                             [Redshift/BigQuery]
                                   |
                             [BI Dashboard]

  Caching Layer:
  [App] --> [ElastiCache Redis] --> (cache miss) --> [RDS]
```

## Key Interview Points

- RDS Multi-AZ = HA (failover), read replicas = scalability (scale reads) — they serve different
  purposes. Don't conflate them.
- For DynamoDB, always discuss access patterns before choosing it — if you need flexible queries,
  use RDS instead.
- Use RDS Proxy at scale to prevent connection exhaustion — Lambda -> RDS without a proxy will
  exhaust DB connections under load.
- Separate OLTP (RDS/DynamoDB) from OLAP (Redshift/BigQuery) — never run analytics queries
  directly on your production database.
- Aurora Global Database: primary in one region, read-only replicas in up to 5 others with ~1s
  replication lag — good for global read-heavy workloads.
- Mention automated backups and point-in-time recovery in any design with persistent data.
