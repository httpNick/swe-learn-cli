---
title: "Search & Indexing (Elasticsearch, Autocomplete, Geospatial)"
tags: [search, elasticsearch, inverted-index, geospatial, autocomplete, full-text-search, tf-idf, geohash]
difficulty: medium
estimated_time: 20min
---

## Overview

Search is one of the most common subsystems in large-scale products, yet it is
frequently under-designed in interviews. The core primitive is the inverted index:
a mapping from terms to the set of documents containing them. At write time,
documents are tokenized (split into terms), normalized (lowercased, stemmed,
stop-words removed), and each term is written into a posting list that records
which documents contain it, along with positional and frequency metadata.
At query time, the engine looks up each query term, intersects posting lists,
and ranks results by a relevance score such as TF-IDF or BM25.

Elasticsearch (built on Apache Lucene) is the dominant open-source engine.
It organizes data into indices, each split into a configurable number of
primary shards. Each shard is a self-contained Lucene index. Replicas are
copies of primaries that serve reads and provide fault tolerance. A cluster
of nodes distributes shards according to allocation rules; the coordinating
node fans out queries to all relevant shards, collects partial results, and
merges them. Mapping (the schema) defines field types: keyword (exact match),
text (analyzed full-text), date, geo_point, etc.

Geospatial search adds a spatial dimension: given a point and a radius (or
bounding box), find all entities within that region. Three common indexing
strategies are geohash (encoding lat/lon into a hierarchical string prefix),
quadtree (recursively subdividing the 2D plane into four quadrants), and
R-tree (bounding-rectangle tree, used in PostGIS and spatial databases).
Elasticsearch uses geohash internally for geo_point fields. Proximity queries
("find restaurants within 2 km") are bread-and-butter for ride-sharing, food
delivery, and local search interview problems.

Autocomplete (type-ahead) requires ultra-low latency (< 100 ms) on prefix
matches. A trie (prefix tree) is the classical data structure: each path from
root to leaf spells out a string, and each node can store a popularity score.
Elasticsearch models this with edge n-gram token filters or the completion
suggester, which builds a finite-state transducer (FST) stored in memory.
Ranking by popularity (query frequency, click-through rate) is crucial;
a pure lexicographic completion is almost never acceptable at FAANG scale.

## When to Use

Reach for a dedicated search index whenever your system needs:
- Relevance-ranked full-text search (product catalog, documents, posts)
- Fuzzy/typo-tolerant matching
- Faceted filtering (category + price range + rating simultaneously)
- Geospatial proximity queries at scale
- Sub-100 ms autocomplete on millions of terms

A relational DB with LIKE queries does not scale past millions of rows.
PostgreSQL full-text search (tsvector/tsquery) is viable for moderate scale
(< ~10M documents, single-digit QPS bursts) but lacks the operational tooling,
relevance tuning, and horizontal scalability of Elasticsearch.

Common interview scenarios where this appears:
- Design Google/Bing/DuckDuckGo (web search) -- inverted index at petabyte scale
- Design Uber/Lyft driver matching -- geospatial index for nearby drivers
- Design Google Maps / Yelp local search -- geohash proximity + ranking
- Design Twitter/Instagram search -- real-time inverted index on posts
- Design a search autocomplete system -- trie / completion suggester + popularity ranking
- Design an e-commerce product search -- Elasticsearch with facets and boosting

## Trade-offs & Gotchas

- Index vs. source of truth: the search index is a derived view of the primary
  DB. Always write to the DB first, then sync to the search index asynchronously
  (via CDC, a Kafka topic, or a periodic batch job). This means there is always
  a consistency lag (typically seconds to minutes); acknowledge this in interviews.

- Dual-write pitfall: writing to DB and search index in the same transaction is
  not possible across heterogeneous systems. Use an outbox pattern or CDC
  (e.g., Debezium reading Postgres WAL) to guarantee the sync eventually happens.

- Shard sizing: Elasticsearch recommends shards of 10-50 GB. Too-small shards
  waste memory on per-shard overhead; too-large shards slow down recovery and
  query merge. A common mistake is starting with too many shards (over-sharding).

- Mapping explosions: using dynamic mapping with high-cardinality nested fields
  (e.g., user-defined tags stored as object keys) can create thousands of fields
  in the mapping, causing memory and performance issues. Use explicit mappings.

- Relevance tuning is never "done": TF-IDF/BM25 gives a reasonable baseline, but
  production search always needs boosting (title > body > tags), field weighting,
  function scores (recency, popularity), and A/B testing. Mention this in interviews.

- Geohash precision trade-off: each additional geohash character roughly divides
  the cell area by 32. Geohash length 5 covers ~5 km x 5 km; length 7 covers
  ~150 m x 150 m. Querying a cell requires also querying the 8 surrounding cells
  to avoid edge-of-cell boundary misses.

- Hot-warm-cold tiering: search clusters often have hot nodes (fast SSDs, high
  RAM, recent data), warm nodes (cheaper HDDs, older data), and cold/frozen nodes
  (object storage, infrequently queried). Index Lifecycle Management (ILM) in
  Elasticsearch automates rolling indices through these tiers.

- Autocomplete ranking drift: if you rank purely by global query frequency, new
  or niche terms never surface. Add recency decay and personalization signals
  where appropriate.

- Replication lag on primary failure: when a primary shard fails and a replica
  is promoted, in-flight indexing operations may be lost. Elasticsearch's
  sequence numbers and global checkpoints bound the data loss window.

## Architecture Diagram

Full-text search with async sync from primary DB:

```
  Writes:
  --------
  [Client]
     |
     v
  [App Server]
     |
     |-- (1) Write to primary DB (Postgres / MySQL)
     |
     v
  [Primary DB]
     |
     | CDC (Change Data Capture via Debezium / binlog)
     v
  [Kafka Topic: db-changes]
     |
     v
  [Indexer Service]   <-- consumes events, transforms docs
     |
     v
  [Elasticsearch Cluster]
     |
     +-- [Primary Shard 0]  <--> [Replica Shard 0]
     +-- [Primary Shard 1]  <--> [Replica Shard 1]
     +-- [Primary Shard 2]  <--> [Replica Shard 2]

  Reads:
  ------
  [Client]
     |
     v
  [App Server / Search API]
     |
     v
  [Coordinating Node]  (any ES node can coordinate)
     |
     | fan-out query to all shards
     v
  [Shard 0]  [Shard 1]  [Shard 2]
     |            |           |
     +------------+-----------+
                  |
         merge + rank (BM25)
                  |
                  v
            [Top-K Results]


Inverted Index (per shard, simplified):

  Term          | Posting List (doc_id: freq)
  --------------|-------------------------------
  "apple"       | [doc3:2, doc7:1, doc12:3]
  "iphone"      | [doc3:1, doc7:4]
  "smartphone"  | [doc1:1, doc7:2, doc99:1]

  Query "apple iphone":
    intersect([doc3,doc7,doc12], [doc3,doc7]) -> [doc3, doc7]
    rank by BM25(tf, idf, field_length_norm) -> doc7 > doc3


Geospatial Indexing (geohash):

  lat/lon  -->  geohash string (e.g., "9q8yy")
                   |
                   | hash prefix hierarchy
                   v
  Level 1: "9"       (~5000 km x 5000 km)
  Level 2: "9q"      (~1250 km x 625 km)
  Level 3: "9q8"     (~156 km x 156 km)
  Level 4: "9q8y"    (~39 km x 20 km)
  Level 5: "9q8yy"   (~4.9 km x 4.9 km)  <-- typical driver proximity cell

  Proximity query: compute geohash of query point, fetch all 9 cells
  (target + 8 neighbors) at chosen precision, then apply exact distance filter.


Autocomplete Trie (prefix tree):

  Query prefix: "sta"

           root
           /|\
          s  ...
          |
          t
          |
          a -----> "star"    (score: 9500)
          |  \---> "stack"   (score: 8200)
          |  \---> "stanford" (score: 4100)
          r
          ...

  Each node stores: children map + top-K completions sorted by score.
  Lookup: O(prefix_length). Top-K is pre-computed at insert time.


Hot-Warm-Cold Node Architecture:

  [Hot Nodes]  (NVMe SSD, 64+ GB RAM)
       |  index actively written to; recent data (last 7 days)
       | ILM rollover when index hits size/age threshold
       v
  [Warm Nodes]  (HDD or SATA SSD, 32 GB RAM)
       |  read-only; last 30-90 days; force-merged to 1 segment
       v
  [Cold Nodes / Frozen]  (searchable snapshots on S3/GCS)
       |  rarely queried; on-demand load from object storage
       v
  [Snapshot Repository]  (S3 / GCS)
       -- full backups; also source for cold/frozen tier
```

## Key Interview Points

- Lead with the inverted index: term -> posting list is the foundational data
  structure. Mention tokenization (splitting), normalization (lowercase, stemming),
  and why raw LIKE queries on a DB do not scale.

- Separate the write path from the read path. Primary DB is the source of truth;
  Elasticsearch is a derived index. Sync asynchronously via CDC + Kafka. Explicitly
  acknowledge the consistency lag and state it is acceptable for most search use cases.

- Elasticsearch sharding: each shard is an independent Lucene index. Primary shards
  cannot be increased after index creation without reindexing; plan shard count upfront
  based on expected data volume and 10-50 GB per shard guidance.

- Relevance: BM25 is the default in Elasticsearch (since v5). It improves on TF-IDF
  by normalizing for document length. Mention field-level boosting (_boost, query-time
  boost) and function_score for business signals (recency, popularity, inventory).

- Geospatial: name the three strategies (geohash, quadtree, R-tree) and explain the
  boundary problem with geohash. Always query the 8 neighboring cells, then apply an
  exact Haversine/Euclidean distance post-filter. At Uber scale, an in-memory quadtree
  per city updated in real time is preferable to ES for driver location.

- Autocomplete: trie for conceptual explanation; edge n-grams or the ES completion
  suggester for implementation. Rank by popularity score, not alphabetically. For
  personalization, blend global popularity with user-specific recency signals.

- Operational concerns worth naming at senior level: ILM / hot-warm-cold tiering for
  cost management, shard sizing mistakes (over-sharding), mapping explosions from
  dynamic fields, and index aliases for zero-downtime reindexing.

- Zero-downtime reindex pattern: write to alias pointing at index v1; create index v2
  with new mapping; reindex v1 -> v2; atomically swap alias to v2. Clients are unaware
  of the swap.
