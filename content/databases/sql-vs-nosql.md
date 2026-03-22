---
title: "SQL vs NoSQL"
tags: [sql, nosql, relational, document, key-value, column-store, schema, acid]
difficulty: easy
estimated_time: 15min
---

## Overview

**SQL (relational) databases** organize data into tables with a fixed schema, enforce relationships via foreign keys, and guarantee **ACID** transactions. Every row in a table shares the same columns, and the schema must be defined before inserting data. PostgreSQL, MySQL, and SQLite are the dominant open-source options. SQL excels when your data is structured, relationships between entities matter, and correctness is non-negotiable (financial records, user accounts, inventory).

**NoSQL** is an umbrella term covering four main models: **document stores** (MongoDB, CouchDB — JSON-like documents with flexible schemas), **key-value stores** (Redis, DynamoDB — fast lookups by a single key), **wide-column stores** (Cassandra, HBase — rows with dynamic columns, optimized for time-series and write-heavy workloads), and **graph databases** (Neo4j — nodes and edges for relationship-heavy queries). NoSQL databases trade ACID guarantees and schema rigidity for horizontal scalability, flexible schemas, and higher write throughput.

The choice is rarely binary. Modern systems often use both: PostgreSQL for transactional records, Redis for caching, and Cassandra or DynamoDB for high-volume event or time-series data. The real decision is about data shape, access patterns, consistency requirements, and scale — not brand loyalty.

## When to Use

- **SQL**: structured data with well-defined relationships, complex multi-table queries (`JOIN`s), strong consistency required, team familiar with relational modeling. E-commerce orders, banking, CMS, SaaS apps.
- **Document (MongoDB)**: semi-structured or variable-schema data (product catalogs, user profiles), rapid iteration where schema changes are frequent, nested/hierarchical data that maps naturally to JSON.
- **Key-value (Redis, DynamoDB)**: simple lookup by ID, session storage, caches, feature flags, shopping carts. Access pattern is always by primary key — no ad-hoc querying.
- **Wide-column (Cassandra, HBase)**: massive write throughput, time-series data, IoT sensor streams, activity feeds. Data is always queried by partition key + optional clustering columns.
- **Graph (Neo4j)**: relationship traversal is the primary access pattern — social networks, fraud detection, recommendation engines.

## Trade-offs & Gotchas

- **Schema flexibility is a double-edged sword**: NoSQL's schemaless nature speeds up early development but causes pain at scale when documents drift apart in shape. Enforce a schema at the application layer (e.g., JSON Schema validation, ODM libraries).
- **NoSQL does not mean no transactions**: MongoDB 4.0+ supports multi-document ACID transactions; DynamoDB supports transactions across items. But they are costlier than native SQL transactions, so avoid them in hot paths.
- **JOINs vs denormalization**: NoSQL avoids JOINs by embedding related data or duplicating it across documents/rows. Reads become fast, but updates must touch multiple locations — write amplification and consistency are your new problems.
- **SQL scales horizontally too** — with read replicas, sharding (Citus for Postgres), or NewSQL systems (CockroachDB, Spanner). Don't assume SQL can't scale; it just requires more explicit engineering effort.
- **Indexes are critical in both worlds**: a missing index turns a millisecond SQL lookup into a full table scan. NoSQL systems have the same failure mode — a Cassandra `ALLOW FILTERING` query or a MongoDB collection scan on a large dataset will time out in production.

## Architecture Diagram

```
SQL (Relational):
  users            orders
  +----+------+    +----+---------+--------+
  | id | name |    | id | user_id | amount |
  +----+------+    +----+---------+--------+
  |  1 | Alice | <--| 10 |       1 |  49.99 |
  |  2 | Bob   |    | 11 |       1 | 120.00 |
  +----+------+    +----+---------+--------+
  Schema enforced, JOINs supported, ACID guaranteed

NoSQL Document (MongoDB):
  {
    "_id": "user:1",
    "name": "Alice",
    "orders": [
      { "id": 10, "amount": 49.99 },
      { "id": 11, "amount": 120.00 }
    ]
  }
  Flexible schema, embedded data, no JOINs needed

NoSQL Wide-Column (Cassandra):
  Partition Key: user_id | Clustering Key: event_time
  --------------------------------------------------
  user:1  | 2024-01-01T10:00 | page_view | /home
  user:1  | 2024-01-01T10:05 | click     | buy_btn
  Optimized for: "get all events for user X, ordered by time"
```

## Key Points for Interviews

- State your access patterns first, then choose the database — not the other way around.
- SQL is not slow; unindexed SQL on large tables is slow. NoSQL is not fast; a Cassandra query without a partition key is a cluster scan.
- "NoSQL = no schema" is a myth — you still have an implicit schema enforced in application code.
- Mention denormalization trade-offs: faster reads, more complex writes, risk of inconsistency.
- For FAANG-scale systems, the answer is often polyglot persistence: SQL for source of truth, NoSQL for derived/cached data.
- Know the four NoSQL families and a real product for each. Interviewers will probe whether you know when Cassandra beats MongoDB or vice versa.
