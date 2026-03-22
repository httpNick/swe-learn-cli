---
title: "CQRS & Event Sourcing"
tags: [cqrs, event-sourcing, command-query, event-store, projections, eventual-consistency, microservices]
difficulty: hard
estimated_time: 25min
---

## Overview

**CQRS (Command Query Responsibility Segregation)** separates the write model (**commands** — intents to change state) from the read model (**queries** — asking for current state). In a traditional CRUD system, the same data model and often the same database table serve both reads and writes. CQRS uses different models — and often different data stores — optimized independently for their respective workloads. The write side enforces business rules and maintains consistency. The read side (projections, read models, materialized views) is denormalized and optimized for query access patterns.

**Event Sourcing** takes CQRS further by replacing mutable state with an append-only **event log** as the source of truth. Instead of storing the current state of a record, you store every event that ever happened to it. Current state is derived by replaying events from the beginning (or from a snapshot). An `Order` is not a row with `status = 'shipped'`; it is the sequence of events: `OrderCreated`, `ItemAdded`, `PaymentProcessed`, `OrderShipped`. The event log is immutable — events are never updated or deleted. This makes auditing trivial, enables temporal queries ("what was the state of this order at 2pm yesterday?"), and allows replaying events to rebuild projections or populate new read models.

These two patterns are often used together but are independent. CQRS without event sourcing: separate read/write models using the same underlying DB with read replicas and denormalized views. Event sourcing without CQRS: one unified model built from event replay. Together, they are a powerful combination: commands emit events to an event store, and read-side projectors subscribe to those events and update purpose-built read models (e.g., a Redis cache, an Elasticsearch index, a relational reporting table). Read models are **eventually consistent** with the event log — there is an inherent lag between when a command is processed and when read models are updated.

## When to Use

- **CQRS**: read-heavy systems where reads vastly outnumber writes; systems with many different "views" of the same data (admin dashboard, mobile app, reporting); domain models where write-side complexity (invariants, aggregates) is high.
- **Event Sourcing**: systems requiring a full audit trail (financial, healthcare, compliance); systems that need to replay history to answer ad-hoc questions; event-driven microservices architectures using Kafka or EventBridge; systems where state reconstruction from events provides a competitive advantage (undo/redo, temporal queries).
- **Both together**: complex domain-driven design (DDD) systems with rich aggregate behavior, where the read-side query diversity is high and write-side correctness is paramount.
- **Avoid when**: the domain is simple CRUD with no complex business rules; team lacks distributed systems experience; eventual consistency on reads is unacceptable; audit history is not required.

## Trade-offs & Gotchas

- **Eventual consistency on reads**: after a command is processed and an event is emitted, read models update asynchronously. A user who just placed an order may briefly see their order count as unchanged. This requires explicit UX handling (optimistic UI updates, "your order is being processed" states) and is often a surprise to teams used to CRUD.
- **Event schema evolution is hard**: events are stored forever and must be readable in the future. Adding, renaming, or removing fields in an event schema requires a migration strategy: versioned event types (`OrderShippedV2`), upcasting (transforming old events to new format on read), or immutable event contracts. Never silently change event structure.
- **Projection rebuild cost**: if you add a new read model or fix a bug in a projection, you must replay all historical events from the beginning. For large event stores (billions of events), this can take hours or days. Snapshots (periodic checkpoints of aggregate state) reduce replay time by starting from the snapshot rather than event 0.
- **Complexity tax**: CQRS/ES adds significant infrastructure: event store, message broker (Kafka, RabbitMQ), projectors, snapshot management, eventual consistency handling. This is not appropriate for simple CRUD services.
- **Distributed transaction pitfall**: a command handler should emit events atomically — either the aggregate state changes AND the event is written, or neither. The **Outbox pattern** solves this: write the event to an `outbox` table in the same DB transaction as the aggregate update, then a separate process reliably publishes the event to the broker.
- **Idempotency**: projectors consuming events must be idempotent — if an event is delivered twice (at-least-once delivery guarantees), applying it twice must produce the same result. Track processed event IDs in the read model store.

## Architecture Diagram

```
Traditional CRUD:
  [Client] --> [Service] --> [DB Table] <-- [Client reads same table]
  One model for reads and writes.

CQRS (separate read/write):
  Write Side:                      Read Side:
  [Client]                         [Client]
     |                                |
  [Command]                        [Query]
     |                                |
  [Write Model / Aggregate]        [Read Model]
     |                             (denormalized, cache, search index)
  [Write DB]    ---sync/async-->   [Read DB]

Event Sourcing + CQRS:
  [Client]
     |
  [Command: PlaceOrder]
     |
  [Order Aggregate]
     |--- validate invariants
     |--- emit: OrderPlaced event
     |
  [Event Store]  <-- append-only log
     |
     +---> [Projector: OrderSummaryView] --> [Read DB: order summaries]
     +---> [Projector: InventoryView]    --> [Redis: stock counts]
     +---> [Projector: AnalyticsStream]  --> [Kafka: analytics pipeline]

Event Log for Order #42:
  t=1  OrderCreated    { order_id: 42, user_id: 7 }
  t=2  ItemAdded       { item: "book", qty: 2 }
  t=3  PaymentReceived { amount: 39.98, method: "card" }
  t=4  OrderShipped    { tracking: "1Z999..." }
  Current state = replay of all events.

Outbox Pattern (atomic event emission):
  BEGIN TRANSACTION
    UPDATE orders SET status='shipped' WHERE id=42
    INSERT INTO outbox(event) VALUES ('{"type":"OrderShipped",...}')
  COMMIT
  [Outbox Poller] reads outbox --> publishes to Kafka --> deletes row
```

## Key Points for Interviews

- CQRS separates write (command) and read (query) models, optimizing each independently. It does not require event sourcing.
- Event sourcing stores events, not state. Current state is derived by replaying events. This makes audit, temporal queries, and projection rebuilds straightforward.
- Eventual consistency is inherent — read models lag behind the event log. Design your UX and API contracts around this.
- Event schema evolution is the hardest operational problem. Mention versioned events and upcasting.
- The Outbox pattern is the correct way to emit events atomically alongside a state change.
- Snapshots reduce event replay cost for large aggregate histories.
- CQRS/ES is a DDD pattern — it shines in domains with rich business logic and audit requirements. Avoid it for simple CRUD services where it adds complexity without payoff.
