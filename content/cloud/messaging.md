---
title: "Messaging & Event Streaming"
tags: [messaging, kafka, sqs, sns, pub-sub, event-streaming, dlq, queues]
difficulty: medium
estimated_time: 25min
---

## Overview

Messaging systems decouple producers from consumers, enabling async processing, load leveling, and
fault tolerance. There are two primary models: message queues (point-to-point) and pub/sub
(fan-out). Understanding when to reach for each — and which technology backs it — is essential
for any distributed system design.

Message queues (SQS, RabbitMQ) deliver each message to exactly one consumer. A producer enqueues
a message; a consumer polls, processes, and deletes it. If processing fails, the message becomes
visible again after the visibility timeout and gets retried. After a configurable number of retries
the message moves to a Dead Letter Queue (DLQ) for inspection. Queues provide at-least-once
delivery — consumers must be idempotent.

Pub/sub (SNS, Google Pub/Sub) delivers each message to all subscribers of a topic. A single
publish fans out to N subscribers simultaneously — useful for triggering multiple independent
workflows from one event (e.g., an order event triggers inventory, billing, and notification
services). SNS → SQS fan-out is a common pattern: SNS fans out, each downstream service has its
own SQS queue for buffering and independent scaling.

Kafka is a distributed log, not just a queue. Messages are persisted to disk with configurable
retention (days, weeks, forever). Consumer groups track their own offset — multiple independent
consumers can read the same stream without interfering. Kafka excels at high-throughput event
streaming, audit logs, CDC (change data capture), and any use case requiring replay. The
tradeoff: operational complexity and higher baseline cost than managed queues.

## When to Use

- **SQS Standard**: async task offloading, decoupling microservices, work queues (image
  processing, email sending, report generation).
- **SQS FIFO**: ordered processing (financial transactions, state machines) — 3,000 msg/s
  throughput limit; higher cost.
- **SNS**: fan-out to multiple subscribers, sending push notifications (mobile, email, SMS).
- **Kafka**: high-throughput event streams, audit logs, CDC pipelines, event sourcing,
  stream processing (Flink, Spark Streaming), any case needing replay or multiple consumers.
- **DLQ**: always configure one — it's your safety net for poison-pill messages.

## Trade-offs & Gotchas

- SQS is at-least-once — duplicate messages are possible. Consumers must be idempotent (use an
  idempotency key stored in a DB or cache).
- SQS visibility timeout must be longer than your max processing time, or messages will be
  re-delivered while still being processed.
- Kafka ordering is per-partition, not global. To order all events for a given entity (e.g.,
  a user), use the entity ID as the partition key.
- Kafka consumer group rebalancing pauses consumption — design for graceful rebalance handling.
- SNS → SQS fan-out adds one hop of latency; for extremely latency-sensitive paths, consider
  direct invocation.
- Dead letter queues need monitoring and alerting — a growing DLQ depth is a leading indicator
  of a downstream processing bug.
- Backpressure: a queue absorbs bursts and prevents your consumers from being overwhelmed, but
  an unbounded queue can grow until it causes memory/storage issues or causes unacceptable lag.

## Architecture Diagram

```
  Task Queue (SQS):
  [Producer] --> [SQS Queue] --> [Consumer Group]
                     |                   |
                 (on failure)     (process & delete)
                     v
                 [DLQ] --> [Alert / Manual Review]

  Fan-out (SNS + SQS):
  [Order Service]
       |
  [SNS Topic: order.placed]
   /         |          \
[SQS]      [SQS]      [SQS]
Inventory  Billing   Notifications

  Kafka Event Stream:
  [Producers] --> [Kafka Topic (N partitions)]
                        |
              +---------+---------+
              |                   |
  [Consumer Group A]   [Consumer Group B]
  (Analytics)          (Notifications)
  (independent offsets, no interference)
```

## Key Interview Points

- Queue = point-to-point (one consumer gets the message). Pub/sub = fan-out (all subscribers get it).
- Use SNS + SQS fan-out pattern when one event must trigger multiple independent services.
- Always configure a DLQ with alerting on depth > 0 — silent message drops are the worst kind
  of bug.
- Kafka for high throughput, replay, multi-consumer, or audit log requirements. SQS for simpler
  async task queues.
- Idempotency is non-negotiable with at-least-once delivery — design consumers to handle
  duplicate messages safely.
- Kafka partition key determines ordering and consumer assignment — choose it based on your
  ordering requirements (e.g., user_id for per-user ordering).
