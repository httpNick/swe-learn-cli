---
title: "Design a Social Media Feed (Twitter/X)"
tags: [feed, fanout, caching, pub-sub, eventual-consistency, social-graph]
difficulty: hard
estimated_time: 60min
companies: [Meta, Twitter/X, Amazon, Google, LinkedIn]
---

## Problem Statement

Design a social media feed where users follow other users and see a reverse-
chronological (or ranked) feed of posts from people they follow. Think Twitter,
Instagram feed, or LinkedIn home feed.

## Clarifying Questions

- How many users? (500M total, 50M DAU)
- Average follows per user? (~200)
- Read/write ratio? (Reads dominate: ~1000:1)
- Reverse-chronological or ranked feed? (Start with chrono; ranking is additive)
- Media support (images/video)? (Text only for design; mention media as extension)
- Maximum followers per user? (Handle celebrities with 10M+ followers — this
  is the core scaling challenge)

## Capacity Estimates

  Posts:  50M DAU * 1 post/day = ~600 writes/sec (peaks 5x)
  Reads:  50M DAU * 10 feed loads/day = ~6,000 reads/sec
  Storage: 600 posts/sec * 140 bytes * 86,400 sec = ~7 GB/day

## High-Level Design

```
[Client]
   |
   v
[API Gateway]
   |           |
   v           v
[Post       [Feed
 Service]    Service]
   |              |
   v              v
[Post DB]    [Feed Cache]    [Feed DB]
   |         (Redis)         (fallback)
   |
   v
[Fanout Service]
(async, via message queue)
```

Core flow:
  Write path: Post → Post DB → enqueue fanout job
  Fanout: push post ID to each follower's feed cache
  Read path: Feed Service reads pre-built feed from cache

## The Core Problem: Fanout

When a user posts, who sees it and how?

### Fanout-on-Write (Push Model)
  - On post creation: write the post ID to every follower's feed cache
  - Read is instant: just fetch the pre-built list from cache
  + Ultra-fast reads
  - Slow/expensive writes for high-follower accounts (celebrity problem)
  - Wasted work if followers are inactive

### Fanout-on-Read (Pull Model)
  - On feed request: fetch posts from everyone the user follows, merge, sort
  + Write is simple and fast
  - Read is slow and expensive at scale (merge N users' timelines)
  - Not viable for users following 1000+ accounts

### Hybrid (recommended at scale)
  - Regular users (<10K followers): fanout-on-write into followers' caches
  - Celebrities (>10K followers): skip write fanout; pull their posts at
    read time and merge with the pre-built feed
  - A "celebrity" flag on the account, checked at post time

```
Post created
     |
     v
Is author a celebrity?
  No  → Fanout Service pushes to all followers' feed caches (async)
  Yes → Skip; feed service pulls celebrity posts at read time
```

## Feed Cache Design

Each user has a Redis list: feed:{user_id} = [post_id_1, post_id_2, ...]

  - Sorted by timestamp descending (newest first)
  - Cap at last 1,000 posts — older posts fetched from Feed DB
  - TTL: evict inactive users' caches after 7 days
  - Fanout writes: LPUSH + LTRIM to maintain cap atomically

Feed read: LRANGE feed:{user_id} 0 49  → 50 post IDs
Then: batch-fetch post content from Post Cache/DB (multi-get).

## Data Model

  posts
  ┌─────────────┬───────────────────────────────────────────┐
  │ post_id     │ BIGINT  PRIMARY KEY (Snowflake ID)        │
  │ author_id   │ BIGINT  NOT NULL                          │
  │ content     │ TEXT    NOT NULL                          │
  │ created_at  │ TIMESTAMP NOT NULL                        │
  └─────────────┴───────────────────────────────────────────┘

  follows
  ┌─────────────┬───────────────────────────────────────────┐
  │ follower_id │ BIGINT  NOT NULL                          │
  │ followee_id │ BIGINT  NOT NULL                          │
  │ PRIMARY KEY (follower_id, followee_id)                  │
  └─────────────┴───────────────────────────────────────────┘

Snowflake IDs encode timestamp — sortable by ID = sortable by time.
No ORDER BY created_at needed; just ORDER BY post_id DESC.

## Deep Dives

### Fanout Queue
  - Post Service publishes to a message queue (Kafka topic: new-posts)
  - Fanout Service consumes, looks up follower list, writes to feed caches
  - Partitioned by author_id for ordering guarantees
  - Lag monitoring: alert if fanout falls >30s behind

### Social Graph Storage
  - follows table sharded by follower_id for efficient "who do I follow?" queries
  - Replicated shard by followee_id for "who follows me?" (fanout lookups)
  - At scale: dedicated graph DB (TAO at Meta) or adjacency list in a KV store

### Feed Ranking (beyond MVP)
  - Instead of pure chronological, score posts by: recency + engagement + affinity
  - Pre-compute scores in the fanout step; store as sorted set (ZADD)
  - Separate ML ranking service consumes feed candidates and re-ranks

### New User / Empty Cache (cold start)
  - No pre-built feed: pull-on-read for first request, then warm the cache
  - Or: background job pre-builds cache when a follow relationship is created

### Read-Your-Own-Write Consistency
  - After posting, user expects to see their post immediately
  - Solution: for the post author, read from Post DB directly for 30 seconds
  - Or: write to their own feed cache synchronously (not async)

## Key Decisions to Highlight

1. Hybrid fanout — write for normal users, pull for celebrities
2. Pre-built feed in Redis for O(1) reads
3. Snowflake IDs — time-sortable, globally unique, no ORDER BY needed
4. Async fanout via Kafka — decouples write path from fan-out latency
5. Feed capped at 1K entries — tail loaded from DB (rarely needed)
