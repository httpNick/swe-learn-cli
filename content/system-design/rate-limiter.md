---
title: "Design a Rate Limiter"
tags: [rate-limiting, redis, distributed-systems, api-design, middleware]
difficulty: medium
estimated_time: 40min
companies: [Google, Amazon, Stripe, Cloudflare, Microsoft]
---

## Problem Statement

Design a rate limiter that restricts how many requests a client can make to an
API within a time window. For example: max 1,000 requests per user per hour.
Throttled requests receive a 429 Too Many Requests response.

## Clarifying Questions

- Per-user, per-IP, or per-API-key limiting? (Assume per user/API key)
- Global limiter or per-service? (Start global, discuss per-service)
- Hard limit (reject) or soft limit (queue/delay)? (Hard reject)
- What consistency is needed? Exact counts or approximate OK? (Approximate OK)
- Distributed — multiple app servers? (Yes)

## High-Level Design

```
[Client]
   |
   v
[Load Balancer]
   |
   v
[Rate Limiter Middleware]  ←→  [Redis Cluster]
   |                |
   v                v
[Allow → API]   [Reject → 429]
```

The rate limiter sits as middleware in the request path, consulting Redis for
the current count before forwarding or rejecting the request.

## Algorithms

### Token Bucket (recommended for most cases)
  - Each user has a bucket with capacity N tokens
  - Tokens refill at a fixed rate (e.g., 10/sec)
  - Each request consumes 1 token; reject if bucket empty
  + Allows bursting up to bucket capacity
  + Smooth, intuitive behavior
  - State per user: {tokens, last_refill_time}

### Sliding Window Counter
  - Divide time into small buckets (e.g., 1-min buckets for a 1-hour window)
  - Sum counts across all buckets in the window
  + More accurate than fixed window
  + Memory efficient with bucketing
  - Slightly complex to implement

### Fixed Window Counter (simplest)
  - Count requests in the current time window (e.g., current minute)
  - Reset counter at window boundary
  + Simple: just increment a key with TTL
  - Allows 2x the limit at window boundaries (thundering herd)

### Leaky Bucket
  - Requests enter a queue; processed at fixed rate
  + Smooths out bursty traffic
  - Adds latency; poor fit for interactive APIs

## Implementation with Redis

Token bucket using Redis + Lua (atomic):

```
-- Lua script executed atomically on Redis
local tokens = tonumber(redis.call("GET", key) or capacity)
local now = tonumber(ARGV[1])  -- current timestamp (ms)
local last = tonumber(redis.call("GET", key..":ts") or now)
local elapsed = (now - last) / 1000
tokens = math.min(capacity, tokens + elapsed * refill_rate)
if tokens >= 1 then
  tokens = tokens - 1
  redis.call("SET", key, tokens)
  redis.call("SET", key..":ts", now)
  return 1  -- allowed
else
  return 0  -- rejected
end
```

Why Lua? Redis executes Lua scripts atomically — no race conditions between
the read-check-write steps without needing distributed locks.

## Sliding Window with Redis (alternative)

  ZADD user:{id}:requests <timestamp> <request_id>
  ZREMRANGEBYSCORE user:{id}:requests 0 <window_start>
  count = ZCARD user:{id}:requests
  if count < limit: allow; else: reject

Each request is a sorted set entry. TTL the key at window_size to auto-clean.
Accurate but higher memory per user (~O(requests_in_window)).

## Response Headers

Always return rate limit info to clients:

  X-RateLimit-Limit: 1000
  X-RateLimit-Remaining: 743
  X-RateLimit-Reset: 1711930800   (Unix timestamp when window resets)
  Retry-After: 3600               (seconds, only on 429)

## Deep Dives

### Distributed Consistency
  - Each app server checks Redis independently — eventual consistency is fine
    for most rate limiting use cases (a few extra requests through is OK)
  - For strict limits (billing, security): use Redis cluster with WAIT for
    replication, or use a single Redis primary for the limiter

### Multiple Limits (burst + sustained)
  - Apply two limiters in series: e.g., max 100 req/min AND max 1000 req/hour
  - Use separate Redis keys per window size: user:123:min, user:123:hour

### Where to Place the Limiter
  Option A — API Gateway: single enforcement point, easy to manage
  Option B — Middleware in each service: more granular, per-service limits
  Option C — Client SDK: soft limits, not enforceable

### Handling Redis Failure
  - If Redis is unavailable: fail open (allow requests) to avoid outage
  - Log the failure and alert — do not silently degrade
  - Use Redis Sentinel or Cluster for HA

### Rate Limiting by IP vs User
  - IP limiting: cheap but easy to bypass with proxies
  - User/API key limiting: more accurate, requires auth at the limiter layer
  - Layer both: IP limit unauthenticated, user limit authenticated

## Key Decisions to Highlight

1. Token bucket for its burst tolerance and natural feel
2. Redis + Lua for atomic check-and-decrement without distributed locks
3. Fail open on Redis failure — availability over strict enforcement
4. Return X-RateLimit-* headers so clients can self-throttle
