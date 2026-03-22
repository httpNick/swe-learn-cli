---
title: "Design a File Storage Service (e.g., Dropbox)"
tags: [file-sync, versioning, conflict-resolution, deduplication, object-storage, delta-sync]
difficulty: hard
estimated_time: 60min
companies: [Google, Microsoft, Dropbox, Box, Amazon]
---

## Problem Statement

Design a cloud file storage and sync service where users can store files, access
them from multiple devices, and sync changes automatically across all devices.

## Clarifying Questions

Ask these before designing:

- Scale? (Assume 500M users, 1B files stored)
- File size limits? (Assume up to 5 GB per file)
- How many devices per user? (Assume ~3 devices average)
- Collaboration / shared folders? (Yes — mention, don't fully design)
- Offline editing and conflict resolution? (Yes — important design constraint)
- Versioning? (Yes — last 30 versions retained)

## Capacity Estimates

  Storage:  500M users * 10 GB avg = 5 EB total
  Upload:   50M daily active users * 1 upload/day * 1 MB avg = 50 TB/day
  Metadata: 1B files * 500 bytes = 500 GB metadata DB
  Sync:     Each upload may need to notify ~3 other devices = 150M sync events/day

## High-Level Design

```
[Client App (Device A)]
    |
    | 1. Detect file change (file watcher)
    | 2. Split into chunks
    | 3. Upload changed chunks only
    v
[Upload Service] ──> [Object Storage (S3)]
    |
    v
[Metadata Service] ──> [Metadata DB (SQL)]
    |
    v
[Notification Service] ──> [Client App (Device B, C)]
                                |
                                v
                        [Download changed chunks]
                        [Reassemble file locally]
```

## Chunking Strategy

Files are split into fixed-size or content-defined chunks (~4–8 MB):

**Why chunking?**
  - Resume interrupted uploads (only re-upload failed chunks)
  - Delta sync: only changed chunks need to be re-uploaded (critical for large files)
  - Deduplication: identical chunks shared across users/files

**Content-Defined Chunking (Rabin fingerprinting):**
  - Chunk boundaries determined by content, not fixed offsets
  - Inserting 1 byte near the start doesn't invalidate all chunks downstream
  - Better dedup ratio than fixed-size chunking

Each chunk is identified by its SHA-256 hash:
  chunk_id = SHA256(chunk_bytes)

## Deduplication

Before uploading a chunk, the client asks: "does this chunk already exist?"

  Client → Metadata Service: "Does chunk {sha256} exist?"
  Server: yes → skip upload (reference existing chunk)
          no  → proceed with upload

**Cross-user dedup:** two users storing identical files → one copy on disk.
  This can raise privacy concerns; some services only dedup within a single user.

**Delta encoding for large files:**
  - Track which chunks changed since last sync
  - Upload only the diff: a 1 GB video file with a 1 MB edit → upload 1 MB, not 1 GB

## Metadata Service

Stores file tree, versions, chunk manifests:

  files
  ┌──────────────────┬───────────────────────────────┐
  │ file_id          │ UUID        PRIMARY KEY        │
  │ user_id          │ UUID                           │
  │ path             │ TEXT        (virtual path)     │
  │ name             │ VARCHAR                        │
  │ size_bytes       │ BIGINT                         │
  │ checksum         │ VARCHAR     (whole-file hash)  │
  │ current_version  │ INT                            │
  │ created_at       │ TIMESTAMP                      │
  │ updated_at       │ TIMESTAMP                      │
  └──────────────────┴───────────────────────────────┘

  file_versions
  ┌──────────────────┬───────────────────────────────┐
  │ file_id          │ UUID                           │
  │ version          │ INT                            │
  │ chunk_ids        │ TEXT[]      (ordered list)     │
  │ created_at       │ TIMESTAMP                      │
  └──────────────────┴───────────────────────────────┘

  chunks
  ┌──────────────────┬───────────────────────────────┐
  │ chunk_id         │ VARCHAR     (SHA256 hash)      │
  │ storage_url      │ TEXT        (S3 key)           │
  │ size_bytes       │ INT                            │
  └──────────────────┴───────────────────────────────┘

## Sync Notifications

When a file changes on Device A, Devices B and C must sync:

1. Metadata Service publishes a file change event (user_id, file_id, new_version)
2. Notification Service fans out to all devices registered for that user
3. Each device receives the event via WebSocket or long-polling connection
4. Device fetches the new version's chunk list from Metadata Service
5. Device downloads only chunks it doesn't already have locally

Use a message broker (Kafka) between Metadata and Notification services for
reliability and decoupling.

## Deep Dives

### Conflict Resolution
Two devices edit the same file while offline:
  - Server detects conflict when Device B uploads v2 based on v1, but v2 already exists
  - Strategy: create a conflict copy (e.g., "report (Nick's conflicted copy 2026-03-21).docx")
  - Both versions preserved; user resolves manually
  - Operational transform (OT) or CRDTs for real-time collaborative editing
    (Google Docs approach — much more complex)

### Client File Watcher
  - OS-level file system events (inotify on Linux, FSEvents on macOS, ReadDirectoryChanges on Windows)
  - Debounce rapid changes (wait 500ms after last event before syncing)
  - Maintain local manifest: file_path → {checksum, last_synced_at}
  - On startup: compare local manifest against server state to catch offline changes

### Large File Upload
  - Multipart upload: upload chunks in parallel to maximize throughput
  - Each chunk uploaded independently; server reassembles the manifest
  - Chunked upload is resumable: restart at last failed chunk

### Bandwidth Optimization
  - Compress chunks before upload (gzip/zstd for text files)
  - Throttle uploads to avoid saturating user's connection (configurable limit)
  - Sync lower-priority files in background; user-accessed files get priority

### Versioning & Deletion
  - Soft delete: move to trash with 30-day retention before permanent removal
  - Versions: keep last 30 versions; background job prunes older versions
  - Chunks from deleted versions are reference-counted; GC when count drops to 0

## Key Decisions to Highlight

1. Chunking with SHA-256 IDs — enables deduplication and delta sync in one mechanism
2. Upload-before-metadata: store chunks first, then commit metadata — crash-safe
3. Conflict copy strategy — simple, user-understandable, preserves no data loss
4. Long-poll/WebSocket sync notifications — near-real-time sync without polling every N seconds
