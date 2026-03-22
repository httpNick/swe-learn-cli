---
title: "Caching"
tags: [caching, redis, memcached, cache-aside, write-through, eviction, ttl]
difficulty: medium
estimated_time: 25min
---

## Overview

Caching stores frequently accessed data in fast memory to reduce latency and offload pressure from
databases and downstream services. A well-designed cache can serve 80–95% of reads without hitting
the database, dramatically improving throughput and reducing cost. The tradeoff is cache
invalidation complexity — "there are only two hard things in computer science: cache invalidation
and naming things."

Redis and Memcached are the two dominant in-memory caches. Memcached is simpler: a pure LRU cache
with string values, multi-threaded, and no persistence. Redis is a full data structure server:
strings, hashes, sorted sets, lists, bitmaps, pub/sub, Lua scripting, and optional persistence
(RDB snapshots, AOF logging). Redis supports clustering and replication; Memcached scales through
consistent hashing across independent nodes. For most new systems, Redis is the default choice.

Cache placement strategies determine when the cache is populated. Cache-aside (lazy loading) only
fetches data on a cache miss, so the cache only holds hot data. Write-through populates the cache
on every write, ensuring cache and DB are always in sync, but wastes memory on cold data. Write-
behind (write-back) writes to cache first and flushes to DB asynchronously — fast writes but
durability risk if the cache fails before the flush.

Multi-level caching adds an in-process cache (L1) in front of Redis (L2) and a CDN edge cache (L3).
Each layer reduces latency further but adds consistency complexity — invalidating across layers
requires careful coordination.

## When to Use

- **Cache-aside**: default for read-heavy workloads. Works well when cache misses are acceptable
  and data is read far more often than written.
- **Write-through**: when you need the cache always warm and can afford the write latency penalty
  (double-write to cache + DB).
- **Write-behind**: write-heavy workloads where durability can be deferred (analytics counters,
  session data, non-critical updates).
- **Redis sorted sets**: leaderboards, rate limiting (sliding window counters), priority queues.
- **Redis as a session store**: stateless API servers with shared session state.

## Trade-offs & Gotchas

- Cache stampede (thundering herd): many requests miss the cache simultaneously (e.g., after TTL
  expiry) and hammer the DB. Mitigate with: mutex locking on cache miss, probabilistic early
  expiry (refresh before TTL expires), or background refresh.
- Cache inconsistency: with cache-aside, a write updates the DB but the cache still serves stale
  data until TTL expires. For strong consistency, invalidate or update the cache key on every
  write.
- Eviction policies: LRU (evict least recently used) is the most common. LFU (least frequently
  used) is better when access frequency matters more than recency. TTL-based expiry handles
  time-sensitive data. allkeys-lru is Redis's recommended policy for pure caches.
- Redis persistence trades memory for durability — AOF logging incurs write amplification.
  Disable persistence if Redis is a pure cache (data can be re-populated from DB on restart).
- Cache key design: include all dimensions that affect the result (user_id, locale, version) to
  avoid serving wrong data. Namespace keys to prevent collisions (e.g., "user:123:profile").
- Memory is finite — set maxmemory and an eviction policy, or Redis will OOM and crash.

## Architecture Diagram

```
  Cache-Aside (Lazy Loading):
  [App]
    |
    +--> [Redis] -- hit --> return value
    |         \-- miss
    |          \--> [DB] --> populate cache --> return value

  Write-Through:
  [App] --> [Cache] --> [DB]  (both writes happen synchronously)

  Write-Behind:
  [App] --> [Cache] --> return (fast)
                  |
             [Async Worker] --> [DB] (eventual flush)

  Multi-Level Cache:
  [Request]
      |
  [L1: in-process HashMap] -- hit --> <1ms
      |-- miss
  [L2: Redis Cluster]       -- hit --> ~1ms
      |-- miss
  [L3: CDN Edge]            -- hit --> ~10ms
      |-- miss
  [Origin DB/API]                    ~50-200ms

  Eviction Policies:
  LRU    -- general purpose, default
  LFU    -- better for skewed access patterns
  TTL    -- time-sensitive data (auth tokens, rate limit windows)
  Random -- simple, low overhead (rare)
```

## Key Interview Points

- Cache-aside is the default; explicitly discuss invalidation strategy (TTL vs active invalidation).
- Cache stampede is the #1 cache failure mode — mention mutex or probabilistic early expiry.
- Redis >> Memcached for new systems: richer data structures, persistence options, cluster mode.
- Always set a TTL — unbounded cache growth leads to eviction storms or OOM.
- Write-behind is fast but dangerous: a Redis failure before the async flush = data loss. Only use
  for non-critical data or when the DB write can be reconstructed.
- Mention cache key namespacing and versioning to handle cache invalidation on deploys.
