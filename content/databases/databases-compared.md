---
title: "Databases Compared"
tags: [postgresql, mysql, mongodb, cassandra, dynamodb, redis, hbase, database-selection]
difficulty: medium
estimated_time: 25min
---

## Overview

Choosing a database is one of the highest-leverage architectural decisions. The right database fits your data model, access patterns, consistency requirements, and operational constraints. The wrong one becomes the root cause of every major incident. This guide compares the most common databases you will encounter in FAANG/MANGA system design interviews.

**PostgreSQL** is the gold-standard open-source relational database. MVCC for non-blocking reads, full ACID transactions, advanced indexing (B-tree, GIN, GiST, BRIN), JSONB for semi-structured data, and a rich extension ecosystem (TimescaleDB for time-series, PostGIS for geospatial, Citus for sharding). Scales well vertically; read replicas for read scale; Citus or manual sharding for write scale. Default choice for new transactional applications.

**MySQL (InnoDB)** is the dominant web-application database. Similar to PostgreSQL in capabilities but with a simpler replication story (binlog-based, widely supported by managed services). Slightly weaker on SQL standards compliance and advanced features. Default choice for many LAMP-stack and high-read web apps. Used at Facebook (with heavy customization), GitHub, and Twitter at scale.

**MongoDB** is the leading document database. Schema-flexible JSON documents, powerful aggregation pipeline, support for multi-document ACID transactions (v4.0+), and horizontal sharding (MongoDB Atlas). Excels at hierarchical and polymorphic data models. Common pitfall: treating it as a relational DB and ending up with deeply nested documents and no indexes. Best fit: product catalogs, user profiles, content management, rapid-iteration applications.

## When to Use

- **PostgreSQL**: transactional workloads, complex queries with JOINs, financial systems, SaaS apps, anything where correctness matters and you want SQL. First choice for greenfield projects.
- **MySQL**: web apps already on the MySQL ecosystem, read-heavy workloads with simple queries, teams with existing MySQL expertise.
- **MongoDB**: flexible/variable schemas, document-oriented data, when the app iterates on schema frequently, when you want to avoid joins by embedding related data.
- **Cassandra**: massive write throughput (millions of writes/sec), time-series data, wide-row data (activity feeds, sensor data), always-on availability across datacenters. Write path is very fast; read path requires careful data modeling.
- **DynamoDB**: fully managed AWS key-value/document store, single-digit millisecond latency at any scale, serverless/ops-free operations, pay-per-request pricing. Best for: AWS-native apps with predictable key-value access patterns, gaming, IoT, session stores.
- **Redis**: in-memory data structure store. Use as a cache (the primary use case), session store, rate limiter, leaderboard (sorted sets), pub/sub bus, distributed lock. Not a primary database (data fits in RAM; persistence is optional).
- **HBase**: wide-column store on top of HDFS/Hadoop. Random read/write on very large datasets (petabyte scale), Hadoop ecosystem integration. Used for: time-series, log storage, Facebook Messenger (historically). Operationally complex; prefer Cassandra or BigTable for new projects unless you are already deep in the Hadoop ecosystem.

## Trade-offs & Gotchas

- **PostgreSQL vs MySQL**: PostgreSQL has better SQL standards compliance, richer index types, and `JSONB`. MySQL has simpler replication and is more widely supported by legacy tooling. For new systems, choose PostgreSQL.
- **MongoDB schema flexibility**: convenient early on, painful at scale when documents diverge. Use JSON Schema validation or ODM libraries to enforce structure. Aggregation pipelines are powerful but can be opaque.
- **Cassandra data modeling**: design tables around queries, not the domain. Create one table per query pattern. A Cassandra query without a partition key is a full cluster scan — the equivalent of a table scan, at potentially petabyte scale.
- **DynamoDB hot partitions**: if all your traffic hits the same partition key (e.g., one user ID dominates all writes), that partition becomes a bottleneck. Use write sharding (add a random suffix to the key) or distribute load across multiple keys.
- **Redis memory limits**: Redis stores all data in RAM. Set `maxmemory` and an eviction policy. Do not use Redis as a primary store for data that cannot be reconstructed from another source, unless you enable AOF/RDB persistence and size your instances accordingly.
- **HBase operational burden**: requires ZooKeeper, HDFS, region servers — a full Hadoop stack. Operationally expensive. Google Cloud Bigtable and HBase-compatible managed services reduce this burden.

## Architecture Diagram

```
Quick Reference Matrix:
  Database    | Type           | Model         | Consistency  | Best For
  ------------+----------------+---------------+--------------+--------------------
  PostgreSQL  | Relational     | Tables/SQL    | ACID         | Transactional apps
  MySQL       | Relational     | Tables/SQL    | ACID         | Web apps, LAMP
  MongoDB     | Document       | JSON docs     | Tunable      | Flexible schema
  Cassandra   | Wide-Column    | Rows/columns  | Tunable AP   | High write volume
  DynamoDB    | Key-Value/Doc  | Items/JSON    | Tunable AP   | AWS serverless
  Redis       | In-Memory      | Structures    | None (cache) | Cache, sessions
  HBase       | Wide-Column    | Rows/columns  | CP           | Hadoop ecosystem

Write/Read Throughput Rough Scale:
  Redis       ~1,000,000 ops/sec  (in-memory)
  Cassandra   ~100,000s writes/sec/node (disk, tunable)
  DynamoDB    ~unlimited (managed, auto-scaled)
  PostgreSQL  ~10,000-50,000 writes/sec (single node, NVMe)
  MySQL       ~10,000-50,000 writes/sec (similar)
  MongoDB     ~10,000-50,000 writes/sec (single node)
  HBase       ~10,000-100,000 writes/sec/node

Typical Polyglot Architecture:
  [Source of Truth: PostgreSQL]
        |
        +--> [Redis] (cache, sessions, rate limits)
        |
        +--> [Elasticsearch] (full-text search)
        |
        +--> [Cassandra/DynamoDB] (event/activity data)
        |
        +--> [MongoDB] (product catalog, CMS content)

Cassandra vs DynamoDB:
  Cassandra:  self-hosted or Astra (managed), CQL (SQL-like),
              tunable consistency, cross-region replication built-in,
              no vendor lock-in
  DynamoDB:   AWS-only, managed/serverless, single-table design,
              GSIs for alternate access patterns, Streams for CDC,
              per-request pricing
```

## Key Points for Interviews

- Start system design answers by stating your access patterns, then map them to database types.
- PostgreSQL is the safe default for anything transactional. Justify any deviation.
- Cassandra's golden rule: design your schema around your queries. One table per query pattern.
- Redis is a cache and auxiliary store, not a primary database — unless your data fits in RAM and you're OK with eventual durability.
- DynamoDB single-table design (all entity types in one table, overloaded partition/sort keys) is a powerful pattern that enables complex access patterns with a single managed service.
- Know the operational complexity order: DynamoDB (managed) < MongoDB Atlas < Cassandra (self-hosted) < HBase (Hadoop required).
- In a FAANG interview, saying "I'd use DynamoDB for X because..." is fine. Also know why you might not (vendor lock-in, cost at scale, limited query flexibility without GSIs).
- Polyglot persistence is common at scale — be ready to explain why you'd use multiple databases in one system and how you'd keep them in sync (CDC, event streaming, dual writes with reconciliation).
