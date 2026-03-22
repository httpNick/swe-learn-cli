---
title: "Database Indexing"
tags: [indexing, b-tree, hash-index, composite-index, covering-index, query-optimization]
difficulty: medium
estimated_time: 20min
---

## Overview

An **index** is a separate data structure that the database maintains alongside a table to speed up lookups. Without an index, finding rows requires a **full table scan** — O(n) in the number of rows. With an index, lookups become O(log n) for tree-based indexes or O(1) amortized for hash indexes. The cost is additional storage and write overhead: every `INSERT`, `UPDATE`, or `DELETE` must also update every relevant index.

The **B-tree index** is the default in virtually every relational database (PostgreSQL, MySQL InnoDB, SQLite). A B-tree is a self-balancing tree where each node holds multiple keys and pointers. Leaf nodes store the actual indexed values plus a pointer (row ID or primary key) to the heap row. B-trees support equality lookups (`WHERE id = 5`), range scans (`WHERE age BETWEEN 20 AND 30`), prefix matching (`WHERE name LIKE 'Al%'`), and `ORDER BY` without an extra sort step. Tree height stays shallow (typically 3–4 levels) even for tables with hundreds of millions of rows.

**Hash indexes** map each key to a bucket using a hash function. Lookups are O(1) but hash indexes only support equality — no range queries, no sorting. PostgreSQL supports explicit hash indexes; MySQL's MEMORY engine uses hash indexes by default. In practice, B-trees are so fast for equality that hash indexes offer marginal benefit and are rarely used in production OLTP systems. Redis and in-memory hash tables are the natural home for hash-based lookups.

**Composite indexes** (multi-column indexes) cover multiple columns in a defined order. A composite index on `(last_name, first_name)` can satisfy queries filtering on `last_name` alone or on both columns, but **not** on `first_name` alone — the **leftmost prefix rule**. Ordering columns by selectivity (most selective first) is a common heuristic, but access pattern always wins: put the equality columns first, then range columns. **Covering indexes** include all columns a query needs, so the database never touches the main table (heap) at all — the index is the answer. This eliminates the extra random I/O of the row fetch.

## When to Use

- **B-tree**: almost always — range queries, sorting, prefix matching, foreign key lookups. The safe default.
- **Hash index**: in-memory tables with pure equality lookups and no range queries. Rare in disk-based OLTP.
- **Composite index**: when queries filter or sort on multiple columns together. Analyze your most frequent slow queries with `EXPLAIN` / `EXPLAIN ANALYZE`.
- **Covering index**: high-frequency read queries on large tables where you want to eliminate all heap I/O. Common in reporting queries that select a small, known set of columns.
- **Partial index** (PostgreSQL): index only rows matching a condition — e.g., `WHERE status = 'pending'`. Keeps the index small and fast for a common selective query.
- **Expression/functional index**: index the result of a function — e.g., `LOWER(email)` for case-insensitive lookups.

## Trade-offs & Gotchas

- **Index bloat**: every write updates all indexes on a table. A table with 10 indexes pays a 10x write amplification penalty for inserts. Keep indexes lean; drop unused ones.
- **Index selectivity**: an index on a boolean column (`is_active`) with 99% `true` is nearly useless — the optimizer will prefer a table scan. High-cardinality columns (user IDs, timestamps, emails) benefit most from indexing.
- **Leftmost prefix rule**: a composite index on `(a, b, c)` is usable for queries filtering on `a`, `(a, b)`, or `(a, b, c)` — but not `b` alone or `(b, c)`. This is one of the most common indexing mistakes in interviews.
- **Index-only scans vs heap fetches**: even with an index, if the query needs columns not in the index, the DB must fetch the heap row (random I/O). Covering indexes eliminate this entirely.
- **Stale statistics**: the query planner uses table statistics to choose between index scan and sequential scan. Run `ANALYZE` (Postgres) or `OPTIMIZE TABLE` (MySQL) after large data loads to keep statistics fresh. A planner that thinks a table has 1,000 rows when it has 10 million will make catastrophically bad plans.
- **Index on foreign keys**: in PostgreSQL, foreign key columns are **not** automatically indexed — unlike MySQL InnoDB. Missing FK indexes cause full child-table scans on every parent row delete, which is a common performance pitfall.

## Architecture Diagram

```
B-Tree Index on users.last_name:
                  [M]
                 /    \
          [F..L]        [N..Z]
          /    \         /    \
      [F][G] [H..L]   [N][O] [P..Z]
        |       |       |       |
      rows    rows    rows    rows   <-- leaf nodes: (key, row_ptr)

  Supports: equality, range, sort, prefix (LIKE 'Sm%')
  Does NOT help: LIKE '%mith' (leading wildcard)

Composite Index (last_name, first_name):
  Query: WHERE last_name = 'Smith'              -> uses index (prefix match)
  Query: WHERE last_name = 'Smith'
           AND first_name = 'Alice'             -> uses index (full match)
  Query: WHERE first_name = 'Alice'             -> CANNOT use index (skips prefix)

Covering Index Example:
  Table: orders(id, user_id, status, amount, created_at)
  Index: CREATE INDEX idx_cover ON orders(user_id, status, amount);
  Query: SELECT amount FROM orders WHERE user_id=1 AND status='paid';
  -> Index contains all needed columns: NO heap fetch (index-only scan)

Hash Index:
  Key --> hash(key) --> bucket --> value
  O(1) lookup. No range, no sort.
```

## Key Points for Interviews

- Know the leftmost prefix rule cold — it comes up constantly. A composite index on `(a, b)` does not help a query filtering only on `b`.
- Default to B-tree. Only reach for other index types when you have a specific justification.
- A covering index is a powerful optimization: frame it as "can I make the index the entire answer to the query?"
- `EXPLAIN ANALYZE` is your best friend — always mention it when discussing query optimization. Seq Scan on a large table = missing index.
- Index writes cost money. For write-heavy tables, fewer, more strategic indexes beat many narrow ones.
- Partial indexes (filtering on a condition) keep index size small — great for soft-delete patterns (`WHERE deleted_at IS NULL`).
- Foreign key columns in PostgreSQL need manual indexes. This is a classic production gotcha.
