---
title: "Design a Chat Application (e.g., WhatsApp)"
tags: [messaging, websockets, real-time, persistence, end-to-end-encryption, queues]
difficulty: hard
estimated_time: 60min
companies: [Meta, Google, Discord, Slack, Microsoft]
---

## Problem Statement

Design a real-time chat application where users can send one-on-one and group
messages, with reliable delivery even when recipients are offline.

## Clarifying Questions

Ask these before designing:

- Scale? (Assume 2B users, 100B messages/day)
- Message types? (Text, images, video — start with text)
- Group chat size limit? (Assume max 256 members)
- Message retention? (Assume indefinite, or 90 days for cost)
- End-to-end encryption required? (Mention as a design concern, don't implement)
- Read receipts (delivered/read indicators)? (Yes)

## Capacity Estimates

  Messages:  100B/day = ~1.15M messages/sec
  Storage:   1.15M/sec * 200 bytes avg = ~230 MB/sec = ~20 TB/day
  Active connections: 500M concurrent users maintaining WebSocket connections

## High-Level Design

```
[Client A]                          [Client B]
    |                                    ^
    | WebSocket                          | WebSocket
    v                                    |
[Connection Service (stateful)]     [Connection Service]
    |                                    |
    v                                    |
[Message Service] ──> [Message Queue] ──> [Delivery Service]
    |                                    |
    v                                    v
[Message DB]                    [Push Notification Service]
(Cassandra)                     (APNs / FCM — for offline)
    |
    v
[Media Service] ──> [Object Storage + CDN]
```

## Real-Time Delivery: WebSockets

Maintain a persistent WebSocket connection per active client:
- Client connects → Connection Service maps user_id → server_node in Redis
- Message sent → Message Service routes to recipient's Connection Service node
- If recipient is online → deliver over WebSocket
- If offline → enqueue in Message Queue → deliver on next connect

WebSocket servers are stateful (hold open connections). Scale horizontally
by sharding users across nodes. Use a connection registry (Redis) to know
which node holds a given user's connection.

## Message Delivery Guarantee

**At-least-once delivery with deduplication:**
1. Sender → server: assign message a unique ID (UUID + timestamp)
2. Server stores message in DB before acknowledging sender
3. Server attempts delivery to recipient
4. Recipient sends ACK → server marks as delivered
5. No ACK within timeout → retry (idempotent due to message ID)

Status flow: SENT → DELIVERED → READ

Read receipts: recipient client sends a lightweight ACK message back to sender
through the same WebSocket infrastructure.

## Message Storage: Cassandra

Why Cassandra for messages?
- High write throughput (1M+ writes/sec)
- Time-series data fits column-family model naturally
- No single point of failure; tunable consistency

Schema (wide-row model):

  messages_by_conversation
  partition key: (conversation_id)
  clustering key: (message_id DESC)  ← newest-first ordering
  columns: sender_id, content, type, sent_at, status

  Query pattern: "fetch last 50 messages for conversation X" → single partition scan

Inbox per user:
  inbox
  partition key: (user_id)
  clustering key: (conversation_id, last_message_at DESC)

## Group Messaging: Fan-out

For a group with N members, sending a message requires N deliveries.

**Fan-out on write (recommended for groups ≤ 256):**
  - On send: write N delivery records (one per member) to the queue
  - Delivery workers process each in parallel
  - Simple, low read latency for recipients

**Fan-out on read (for large groups / channels):**
  - Store one copy of message; each reader queries from their last-read pointer
  - Better write efficiency; more complex read path

For group chats ≤ 256: fan-out on write. For broadcast channels (thousands):
fan-out on read with a cursor-based approach.

## Offline Delivery

When a recipient is offline:
1. Message lands in the Message Queue (Kafka topic per user)
2. Push Notification Service sends a notification via APNs (iOS) or FCM (Android)
3. Notification wakes the app → client connects → pulls pending messages
4. Message Queue retains messages for N days (configurable) for late delivery

## Deep Dives

### Media Messages (Images/Video)
  - Client uploads media directly to object storage via pre-signed URL
  - Server stores only the CDN URL reference in the message record
  - Recipient downloads media from CDN — no media bytes through chat servers
  - Thumbnail generated on upload for preview display

### End-to-End Encryption (E2EE)
  - Each client generates a public/private key pair
  - Server stores public keys only; private keys never leave device
  - Sender encrypts message with recipient's public key
  - Server stores and delivers ciphertext; cannot read content
  - Signal Protocol (used by WhatsApp) handles key exchange and rotation

### Message Search
  - E2EE makes server-side search impossible
  - Client-side search: index plaintext locally after decryption
  - Non-E2EE systems: Elasticsearch index on message content

### Presence (Online/Offline Status)
  - Connection Service publishes presence events to Pub/Sub on connect/disconnect
  - User's contacts subscribe to their presence topic
  - Last-seen timestamp stored in Redis with TTL; background heartbeat refreshes it
  - Privacy: allow users to hide presence status

## Key Decisions to Highlight

1. WebSocket + connection registry — routes server-push to the right node
2. Cassandra wide rows — natural fit for time-ordered message history per conversation
3. At-least-once delivery with ACKs — reliability without complex distributed transactions
4. Fan-out on write for groups ≤ 256 — bounded amplification, simple delivery path
