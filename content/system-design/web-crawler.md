---
title: "Design a Web Crawler"
tags: [distributed-systems, queues, politeness, deduplication, dns, scraping]
difficulty: medium
estimated_time: 45min
companies: [Google, Bing, Amazon, Cloudflare]
---

## Problem Statement

Design a web crawler that systematically browses the internet, downloads web pages,
and stores their content — the foundational component of a search engine index.

## Clarifying Questions

Ask these before designing:

- Scope: general-purpose or domain-specific? (Assume general-purpose)
- How many pages to crawl? (Assume 1B pages, ~10KB avg = 10 PB raw)
- How often to re-crawl (freshness)? (Assume every 2–4 weeks for most pages)
- Handle JavaScript-rendered content? (No — defer to Phase 2)
- Store raw HTML or parsed content? (Both — raw for re-processing, parsed for indexing)

## Capacity Estimates

  Target: 1B pages crawled over 30 days
  Rate:   ~400 pages/sec sustained
  Storage: 1B * 10KB = 10 TB per crawl cycle
  DNS:    ~400 DNS lookups/sec (cache aggressively)

## High-Level Design

```
[Seed URLs]
     |
     v
[URL Frontier / Priority Queue]
     |
     v
[Fetcher Workers] ──> [DNS Cache]
     |
     v
[HTML Parser / Link Extractor]
     |            |
     v            v
[Content Store] [URL Dedup Filter]
(S3 / HDFS)         |
                     v
              [URL Frontier] (loop)
```

Three core loops:
1. Fetch: pull URL from frontier, download page, store raw HTML
2. Parse: extract links and text from stored HTML
3. Schedule: filter duplicate/already-seen URLs, re-enqueue new ones

## URL Frontier

The frontier is more than a queue — it enforces:

**Politeness:** respect robots.txt and crawl delays per domain.
  - Per-domain back-queue with minimum delay (e.g., 1 req/sec per host)
  - Separate priority queue by importance (PageRank estimate, freshness)

**Priority:**
  - High: news sites, frequently-changing pages
  - Low: deep archive pages, rarely-linked content

Implementation: distributed priority queue (Kafka partitioned by host hash, or
Redis sorted sets for small scale).

## Deduplication

Two levels of dedup are required:

**URL dedup:** don't re-fetch the same URL.
  - Bloom filter in memory (~1 GB for 1B URLs at 1% false positive rate)
  - Persistent seen-set in Redis or a key-value store for durability

**Content dedup:** don't store identical pages under different URLs.
  - Compute SimHash (or MD5) of page content
  - Near-duplicate detection with SimHash bit-difference threshold

## robots.txt

Every crawler MUST obey robots.txt:
  1. Fetch `https://host/robots.txt` once per domain, cache for 24h
  2. Check Disallow rules before fetching any URL from that host
  3. Respect Crawl-delay directive
  4. Set a descriptive User-Agent identifying your crawler

Violating robots.txt risks IP bans and legal issues.

## Deep Dives

### Distributed Fetcher Fleet
  - Stateless fetcher workers — scale horizontally
  - Each worker pulls a batch of URLs for a specific host subset (partitioned to
    respect per-domain politeness)
  - Health checks; failed fetches go back on the queue with a retry count

### Handling Traps
  - Spider traps: dynamically generated infinite URL spaces (e.g., calendars)
    - Limit URL path depth (e.g., max 6 levels deep)
    - Detect patterns: URLs with incrementing query params
  - Duplicate content under canonical vs. non-canonical URLs
    - Follow <link rel="canonical"> headers

### Re-crawl Scheduling
  - Track last-modified headers and ETag — use conditional GET (If-Modified-Since)
  - Pages that change often → shorter re-crawl interval (exponential smoothing)
  - Pages with 404 → back off exponentially then drop after N failures

### DNS at Scale
  - ~400 DNS lookups/sec requires a local caching DNS resolver
  - Batch resolve hostnames ahead of fetch time
  - Respect DNS TTL but cache aggressively for known stable domains

## Key Decisions to Highlight

1. URL Frontier partitioned by host — enforces politeness without coordination overhead
2. Bloom filter for URL dedup — O(1) lookup, probabilistic but tunable
3. Separate fetch → parse → schedule pipeline — each stage scales independently
4. robots.txt compliance cached per domain — respect crawl delay, avoid bans
