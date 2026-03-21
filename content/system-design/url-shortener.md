---
title: "Design a URL Shortener"
tags: [hashing, databases, caching, api-design, key-value-store]
difficulty: medium
estimated_time: 45min
companies: [Google, Amazon, Meta, Microsoft]
---

## Problem Statement

Design a URL shortening service like bit.ly or TinyURL. Users paste a long URL
and receive a short alias (e.g., short.ly/aB3x9z) that permanently redirects to
the original URL.

## Clarifying Questions

Ask these before designing:

- Read/write ratio? (Typical: ~100:1 — reads dominate)
- How many URLs shortened per day? (Assume 100M/day)
- How long should short URLs live? (Assume forever unless deleted)
- Should custom aliases be supported? (Yes, as a stretch goal)
- Analytics needed? (Click counts, geo — defer to later)

## Capacity Estimates

  Write:  100M URLs/day = ~1,200 writes/sec
  Read:   10B redirects/day = ~115,000 reads/sec
  Storage: 100M * 365 * 10 years * 500 bytes ≈ 180 TB over 10 years

## High-Level Design

```
[Client]
   |
   v
[API Gateway / Load Balancer]
   |              |
   v              v
[Write Service] [Read Service]
   |              |
   v              v
[SQL DB]      [Redis Cache]
(source of        |
  truth)      [SQL DB] (cache miss fallback)
```

Two services split by concern:
- Write Service: generates short key, stores mapping, returns alias
- Read Service: looks up alias → redirects (cache-first)

## URL Generation: Hashing vs Counter

Option A — MD5/SHA hash (truncated to 6-7 chars):
  + Stateless, no coordination needed
  - Hash collisions require retry logic
  - Hard to guarantee uniqueness at scale

Option B — Base62 counter (recommended):
  + No collisions, predictable
  + 62^7 = ~3.5 trillion unique keys
  - Requires a global counter (use a dedicated ID service or DB auto-increment)
  - Sequential IDs are guessable — shuffle with a bijection if privacy matters

Base62 alphabet: [0-9][a-z][A-Z] → 62 chars → 7 chars gives 3.5T keys

## Database Schema

  urls
  ┌─────────────┬──────────────────────────────────────────────┐
  │ short_key   │ VARCHAR(8)   PRIMARY KEY                     │
  │ long_url    │ TEXT         NOT NULL                        │
  │ created_at  │ TIMESTAMP    NOT NULL                        │
  │ expires_at  │ TIMESTAMP    NULLABLE                        │
  └─────────────┴──────────────────────────────────────────────┘

Single table. A key-value store (DynamoDB, Redis) also works — reads are pure
lookups with no joins needed.

## Caching Strategy

Use cache-aside (lazy loading):
  1. Read Service checks Redis for short_key
  2. Cache hit → return long_url immediately (< 1ms)
  3. Cache miss → query DB, populate cache, return

TTL: set per URL or a sensible default (e.g., 24h for hot links).
Cache ~20% of URLs to serve ~80% of traffic (Pareto principle).

## Deep Dives

### Collision Handling (if using hashing)
  - Append an incrementing suffix and re-hash on collision
  - Or: pre-generate a pool of unique short keys offline and pull from a queue

### Custom Aliases
  - Accept user-provided key, validate uniqueness before storing
  - Charge or rate-limit to prevent abuse

### Redirects: 301 vs 302
  - 301 Permanent: browser caches — fewer requests hit your servers
  - 302 Temporary: every redirect hits your servers — better for analytics
  - Choose 302 if click tracking matters

### Deletion / Expiry
  - Soft delete: set expires_at, filter on read
  - Background job sweeps expired rows nightly

### Scalability
  - Read Service is stateless — scale horizontally behind a load balancer
  - DB read replicas handle read scale
  - Redis cluster for cache scale
  - Write Service bottleneck is the ID generator — use a token bucket service
    or Snowflake-style distributed ID generation

## Key Decisions to Highlight

1. Base62 counter over hashing — no collisions, simpler retry logic
2. Separate read/write services — scale independently
3. Cache-aside with Redis — absorbs 80%+ of read traffic
4. 302 redirect if analytics are in scope, 301 for pure performance
