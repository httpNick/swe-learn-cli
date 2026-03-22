---
title: "CDN: Edge Caching and Content Delivery"
tags: [cdn, edge-caching, cache-invalidation, origin-pull, origin-push, hls, dash, video-streaming]
difficulty: medium
estimated_time: 20min
---

## Overview

A **CDN (Content Delivery Network)** is a globally distributed network of **edge nodes (PoPs — Points of Presence)** that cache content close to users, reducing latency and offloading traffic from origin servers. Instead of every user in Tokyo hitting your origin in us-east-1, a Tokyo edge node serves the cached response. CDNs are the standard solution for serving static assets (images, JS, CSS), large file downloads, and video streams at scale. Major providers: **Cloudflare**, **AWS CloudFront**, **Fastly**, **Akamai**.

**Origin pull** (lazy caching) is the default model: on a cache miss, the edge fetches from origin, caches the response, and serves it to the user. Subsequent requests from the same edge hit the cache. **Origin push** (eager caching) is used for content that must be globally warm before requests arrive — you explicitly push content to edge nodes. Push is common for software releases, large media uploads, or predictable traffic spikes. Most CDNs (CloudFront, Fastly) are pull-based by default; Akamai and some enterprise CDNs support push.

**Cache invalidation** is notoriously hard. The CDN respects `Cache-Control` headers from origin (e.g., `max-age=86400`). To invalidate before TTL expires: (1) create a CloudFront invalidation API call (costs money per path), (2) use **cache-busting** — embed a content hash in the filename (`app.a3f9d2c.js`) so every deploy creates a new URL with no prior cache entry, (3) use surrogate keys / cache tags (Fastly, Cloudflare) to tag multiple cached objects with a logical key and purge by tag atomically.

```
Origin Pull Flow (Cache Miss then Cache Hit):

  [User in Tokyo] -> [Tokyo Edge PoP]
                          |
                     cache miss?
                          |  YES
                    [Fetch from Origin us-east-1]
                          |
                     cache response (TTL=86400)
                          |
                    [Serve to User]  (high latency, 1 time)

  Next request from Tokyo:
  [User in Tokyo] -> [Tokyo Edge PoP]
                          |
                     cache HIT -> serve immediately (low latency)

Cache Hierarchy (some CDNs use 2 tiers):
  [User] -> [L1 Edge PoP] -> [L2 Shield/Mid-Tier] -> [Origin]
  Shield concentrates origin requests, improves cache hit rate

CDN Cache-Control Headers:
  Cache-Control: public, max-age=31536000, immutable
    -> browser + CDN cache for 1 year, never revalidate
  Cache-Control: no-cache
    -> always revalidate with origin (ETag/If-None-Match)
  Cache-Control: no-store
    -> do not cache anywhere (sensitive data)
  Surrogate-Control: max-age=86400
    -> CDN-only TTL (stripped before sending to browser)

Video Streaming: HLS vs DASH:
  HLS  (Apple)  : .m3u8 manifest + .ts  segments (MPEG-TS)
  DASH (MPEG)   : .mpd  manifest + .mp4 segments (fMP4)
  Both: segment content (2-10s chunks) served via CDN
  ABR (Adaptive Bitrate): player selects quality based on bandwidth
    320p -> 480p -> 720p -> 1080p -> 4K
  CDN role: cache segments at edge, reduce origin load,
            deliver segments with low latency for live streams
```

## When to Use

- **Static assets**: Always serve CSS, JS, images, fonts via CDN with long TTLs + cache-busting hashes. `Cache-Control: public, max-age=31536000, immutable`.
- **API responses**: Cacheable GET responses with `Cache-Control: public, max-age=60` work well at CDN level for read-heavy endpoints (product catalogs, leaderboards). Use Vary headers carefully — `Vary: Accept-Encoding` is safe; `Vary: Cookie` essentially disables caching.
- **Video on demand (VOD)**: HLS or DASH segments cached at CDN. Set long TTLs on segments (immutable), short TTL on the manifest file (it changes as new segments are added in live streams).
- **Live streaming**: Low-latency HLS (`LL-HLS`) or CMAF-CTE reduces segment size to <1s. CDN must support short TTLs and efficient origin shielding for live content.
- **DDoS mitigation**: CDN absorbs volumetric attacks at the edge before traffic reaches origin. Enable CDN in front of your origin as a default security measure.
- **Origin shield**: Enable CDN's mid-tier caching layer (CloudFront Origin Shield, Fastly shielding) to collapse origin requests from all edge nodes through a single regional cache — dramatically reduces origin load.

## Trade-offs & Gotchas

- **Cache invalidation timing**: CloudFront invalidations take ~60s to propagate globally. Fastly instant purge is <150ms globally. If consistency matters, design for cache-busting rather than invalidation.
- **Vary header explosion**: `Vary: User-Agent` creates a separate cache entry per browser — effectively disables caching. Never use Vary on high-cardinality headers. `Vary: Accept-Encoding` (for gzip/brotli) is the safe standard.
- **Private vs public content**: CDNs should never cache responses containing user-specific data. Use `Cache-Control: private` for authenticated responses, or move user-specific data to client-side rendering.
- **Geo-blocking and compliance**: CDNs can enforce GDPR geo-restrictions at the edge without hitting origin. CloudFront geo-restriction, Cloudflare Access Rules.
- **SSL/TLS at edge**: CDN terminates TLS at the edge PoP (closer to user = lower handshake latency). The CDN-to-origin connection can be HTTP or HTTPS; use HTTPS for origin-pull to prevent snooping on the backhaul.
- **Warm-up time for new PoPs**: On first deploy or after a cold cache, origin receives full load while CDN populates. Plan for origin capacity to handle 100% traffic, even when CDN hit rate is normally 95%+.
- Cache hit rate is the key CDN metric. A 90% hit rate means origin handles 10% of traffic. A 70% hit rate may not justify CDN costs — review TTLs and caching strategy.

## Key Points for Interviews

- CDN = distributed edge caches; reduces origin load, improves latency, absorbs DDoS.
- Origin pull (lazy) vs origin push (eager, for pre-warming). Pull is default; push for predictable large-content deploys.
- Cache invalidation strategies: TTL expiry, file hash in URL (cache-busting), surrogate keys / cache tags (Fastly/Cloudflare), explicit invalidation API.
- HLS and DASH are segmented streaming formats. CDN caches segments; manifest files have short TTLs for live streams.
- `Cache-Control: public, max-age=31536000, immutable` for versioned static assets. `private` for user-specific responses.
- Origin Shield / mid-tier caching collapses parallel edge misses to a single origin request — critical for high-traffic deploys.
- Cache hit rate is the metric. Low hit rate = wasted CDN cost. Tune TTLs and avoid Vary on high-cardinality headers.
