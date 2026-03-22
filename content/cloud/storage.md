---
title: "Cloud Storage"
tags: [object-storage, block-storage, s3, storage-tiers, data-lake]
difficulty: easy
estimated_time: 20min
---

## Overview

Cloud storage breaks into three fundamental models: object, block, and file. Choosing the wrong
one is a common design mistake — each maps to a different access pattern and consistency model.

Object storage (S3, GCS, Azure Blob) stores data as flat objects identified by a key within a
bucket. There is no directory hierarchy (prefixes simulate one). Objects are accessed over HTTP,
are immutable once written (must overwrite entirely), and scale to virtually unlimited capacity.
S3 achieved strong read-after-write consistency for all operations in December 2020 — no more
eventual consistency caveats for new objects or overwrites.

Block storage (EBS, Persistent Disk) presents as a raw disk attached to a single VM instance.
The OS formats it with a filesystem. Latency is sub-millisecond, making it the right choice for
database storage, boot volumes, and any workload requiring random I/O. Block volumes are bound
to one AZ and typically one instance (multi-attach exists but is complex).

File storage (EFS, Filestore) provides a POSIX-compliant shared filesystem mountable by multiple
instances simultaneously. It is the choice for shared configuration, CMS asset storage, or any
workload where multiple servers need concurrent read/write access to the same directory tree.

## When to Use

- **Object storage**: media files, backups, logs, static website assets, ML training data,
  data lake raw zone. Default choice when no low-latency random I/O is needed.
- **Block storage**: primary storage for relational databases, Kafka brokers, anything the OS
  treats as a disk drive.
- **File storage**: shared application config, home directories, CMS uploads, HPC scratch space.
- **Data lake**: raw + processed analytics data at scale — combine object storage with a metadata
  catalog (Glue, Hive Metastore) for schema-on-read querying via Spark/Athena.

## Trade-offs & Gotchas

- Object storage has higher per-operation latency (5–100ms) vs block storage (<1ms). Never use
  S3 as a database.
- EBS volumes are AZ-scoped — a failure or migration requires a snapshot + restore to another AZ.
- EFS is more expensive than EBS per GB and higher latency; only use it when sharing is required.
- Data lake query costs scale with data scanned — partition by date/region and use columnar formats
  (Parquet, ORC) to minimize scanned bytes.
- Storage tiers (S3 Standard → Infrequent Access → Glacier) reduce cost for cold data but add
  retrieval latency and fees. Use lifecycle policies to automate tiering.
- Large object uploads: use multipart upload for objects > 100MB to improve throughput and
  resilience to network interruptions.

## Architecture Diagram

```
  Write Path (user upload):
  [Client] --> [API Server] --> [Object Storage (S3)]
                                      |
                               [Event Notification]
                                      |
                              [Processing Lambda]
                                      |
                         [Processed Bucket (S3)]

  Data Lake Layers:
  [Raw Zone / S3]          <- ingestion, immutable
       |
  [Curated Zone / S3]      <- cleaned, Parquet format
       |
  [Serving Zone / S3]      <- aggregated, query-optimized
       |
  [Query Engine]           <- Athena / Spark / BigQuery

  Storage by Latency:
  Block (EBS)  <1ms    -- databases, boot volumes
  File  (EFS)  ~1ms    -- shared filesystems
  Object (S3)  5-100ms -- everything else
```

## Key Interview Points

- S3 has strong consistency (since 2020) — safe to read immediately after write.
- Use object storage for anything you'd put in a CDN or serve over HTTP — it's cheap and
  infinitely scalable.
- Block storage is for databases; file storage is for shared mounts; object is for everything else.
- Columnar formats (Parquet) + partitioning dramatically reduce data lake query costs.
- Storage tiers: Standard → Standard-IA (infrequent access) → Glacier (archive). Automate
  with lifecycle rules, not manual management.
- Mention versioning on S3 for audit trails and accidental-deletion protection.
