---
title: "Design YouTube / Video Streaming"
tags: [cdn, video-encoding, streaming, object-storage, caching, transcoding]
difficulty: hard
estimated_time: 60min
companies: [Google, Meta, Netflix, TikTok, Microsoft]
---

## Problem Statement

Design a video streaming platform like YouTube where users can upload videos,
and other users can watch them with low buffering and high availability globally.

## Clarifying Questions

Ask these before designing:

- Upload volume? (Assume 500 hours of video uploaded per minute)
- Scale of playback? (Assume 5M concurrent viewers at peak)
- Supported resolutions? (360p, 720p, 1080p, 4K)
- Monetization / ads in scope? (No — defer)
- Live streaming in scope? (No — focus on on-demand)

## Capacity Estimates

  Upload:   500 hrs/min = ~8 hrs/sec of raw video
  Storage:  1 min of 1080p video ≈ 1.5 GB raw → ~150 MB after encoding
            500 hrs/min * 60 * 150 MB * 5 resolutions ≈ 27 TB/min (with all qualities)
            Realistic: ~2 PB/day stored
  Playback: 5M concurrent * 5 Mbps (avg) = 25 Tbps egress → must use CDN

## High-Level Design

```
[User Browser / App]
        |
        | Upload
        v
[Upload Service] ──> [Object Storage (raw)] ──> [Transcoding Workers]
                                                         |
                                                         v
                                                [Object Storage (HLS segments)]
                                                         |
                                                         v
[User Browser / App] <── [CDN Edge Nodes] <── [Origin Servers]
        |
        | Search / Browse
        v
[API Gateway] ──> [Metadata Service] ──> [Metadata DB (SQL)]
                                    ──> [Search Index (Elasticsearch)]
```

Separation of concerns:
- Upload path: raw → transcode → serve
- Playback path: metadata + CDN-delivered segments
- These are independent — a slow transcoding job never blocks playback

## Video Upload Flow

1. Client requests a pre-signed upload URL from Upload Service
2. Client uploads directly to object storage (S3/GCS) — bypasses app servers
3. Object storage triggers an event → Transcoding Job Queue (SQS/Kafka)
4. Transcoding Workers pull jobs, encode to multiple resolutions using FFmpeg
5. Encoded segments uploaded to CDN-backed object storage
6. Metadata Service updated: video status = "ready", segment manifest stored

Pre-signed URLs avoid routing large binary payloads through your API tier.

## Transcoding Pipeline

Raw video → multiple output formats:
  - Resolutions: 360p, 480p, 720p, 1080p, 4K (where applicable)
  - Format: HLS (HTTP Live Streaming) — splits video into ~10s .ts segments
             with an M3U8 manifest file per resolution
  - Codec: H.264 for compatibility, H.265/AV1 for efficiency at higher res

Worker fleet is horizontally scalable — transcoding is CPU-bound and embarrassingly
parallel. Use spot/preemptible instances for cost efficiency.

## Streaming Protocol: HLS vs DASH

HLS (HTTP Live Streaming):
  + Native support on iOS/Safari
  + Widely supported by CDNs
  - Apple-originated; slightly less flexible than DASH

DASH (Dynamic Adaptive Streaming over HTTP):
  + Open standard
  + Better adaptive bitrate flexibility
  - Less native browser support

Adaptive Bitrate (ABR): player monitors bandwidth and switches quality level
on-the-fly by fetching segments from different M3U8 playlists. No server-side
state needed — the player decides.

## CDN Strategy

CDN is the most critical component for playback at scale.

  - Video segments are static files — perfect for CDN edge caching
  - Long TTL (hours/days) for encoded segments; they never change
  - Short TTL for M3U8 manifests (may update for live or newly processed videos)
  - Multi-CDN: use 2+ CDN providers for redundancy and cost negotiation
  - Cache hit rate target: > 95% for popular videos; long-tail videos served from origin

Popular videos: top 1% of videos serve ~80% of traffic — these stay hot in CDN.
Long-tail: rare videos miss CDN, served directly from object storage via origin pull.

## Metadata Service

Stores video metadata separately from video bytes:

  videos
  ┌──────────────┬──────────────────────────────────┐
  │ video_id     │ UUID        PRIMARY KEY           │
  │ user_id      │ UUID        NOT NULL              │
  │ title        │ VARCHAR     NOT NULL              │
  │ description  │ TEXT                              │
  │ status       │ ENUM(processing, ready, failed)   │
  │ manifest_url │ TEXT        (CDN URL to M3U8)     │
  │ duration_sec │ INT                               │
  │ created_at   │ TIMESTAMP                         │
  └──────────────┴──────────────────────────────────┘

Search: Elasticsearch index on title, description, tags — updated asynchronously
after video is ready.

## Deep Dives

### Resumable Uploads
  - Large video files need resumable upload support (network drops)
  - Use chunked upload protocol: client uploads in 5–50 MB chunks
  - Server tracks received chunks; client can resume from last successful chunk
  - GCS and S3 support this natively

### Deduplication
  - Perceptual hash (pHash) of video frames detects re-uploads of same content
  - Can skip re-transcoding if identical content already exists
  - Important for copyright enforcement (Content ID-style system)

### Thumbnail Generation
  - Extract keyframes during transcoding at 1s, 5s, 10s intervals
  - Store as images in object storage, serve via CDN
  - Auto-select or allow creator to choose

### Geographic Replication
  - Replicate popular video segments to CDN PoPs closest to user density
  - Hot video prediction: if a video trends, proactively push to more edge nodes
  - Cold storage: videos with < 100 views/month moved to Glacier/Coldline

## Key Decisions to Highlight

1. Pre-signed upload URLs — offload binary upload traffic from API servers
2. HLS segmentation — enables adaptive bitrate and CDN edge caching
3. CDN-first delivery — without CDN, egress at 5M concurrent viewers is impossible
4. Async transcoding pipeline — upload and playback are decoupled; failure is isolated
