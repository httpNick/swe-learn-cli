---
title: "Design a Notification System"
tags: [push-notifications, message-queues, fan-out, user-preferences, apns, fcm]
difficulty: medium
estimated_time: 45min
companies: [Google, Meta, Amazon, Uber, LinkedIn]
---

## Problem Statement

Design a notification system that sends notifications to users across multiple
channels (push, email, SMS) triggered by various application events.

## Clarifying Questions

Ask these before designing:

- Channels supported? (Push notifications, email, SMS — all three)
- Scale? (Assume 10M notifications/day, spikes up to 1M/hr during events)
- Latency requirement? (Push: < 5s; email: best-effort; SMS: < 30s)
- User preferences respected? (Yes — users can opt out per channel/type)
- Templating and personalization? (Yes — variable substitution in templates)
- Analytics / delivery tracking? (Yes — delivered, opened, clicked)

## Capacity Estimates

  10M notifications/day = ~115/sec average
  Peak: 1M/hr = ~277/sec during marketing campaigns
  Fanout: a single event (e.g., new follower) → 1 notification
          a broadcast (e.g., marketing blast) → 100M notifications

## High-Level Design

```
[Event Sources]
(App Services, Scheduled Jobs, Admin Campaigns)
     |
     v
[Notification Service / API]
     |
     v
[Validation + User Preference Check]
     |
     v
[Message Queue (Kafka)]
     |
  ┌──┴──────────────┬───────────────┐
  v                 v               v
[Push Worker]   [Email Worker]  [SMS Worker]
  |                 |               |
  v                 v               v
[APNs / FCM]   [SendGrid/SES]   [Twilio]
  |                 |               |
  v                 v               v
         [Delivery Tracker DB]
```

## Event Ingestion

Notification triggers arrive from many sources:
- Application services (user liked your post, your order shipped)
- Scheduled jobs (weekly digest emails, daily reminders)
- Admin campaigns (marketing blasts, product announcements)

The Notification Service exposes an internal API:

  POST /notifications
  {
    "user_id": "uuid",
    "type": "ORDER_SHIPPED",
    "template_id": "order-shipped-v2",
    "data": { "order_id": "123", "carrier": "UPS" },
    "channels": ["push", "email"]
  }

## User Preferences

Before enqueuing, check if the user wants this notification:
- User opted out of this notification type?
- User opted out of this channel?
- User in do-not-disturb hours?
- User's device token still valid?

Store preferences in a fast read store (Redis or a SQL table with read replicas).
Invalid/expired tokens should be cleaned up proactively (APNs/FCM return errors
for invalid tokens — act on these responses).

## Message Queue: Kafka

Use separate Kafka topics per channel:
  notifications.push   → consumed by Push Workers
  notifications.email  → consumed by Email Workers
  notifications.sms    → consumed by SMS Workers

Benefits:
  - Channel workers scale independently (push volume >> SMS volume)
  - Backpressure: if email provider is slow, email queue absorbs the spike
  - Retry: failed deliveries requeued with exponential backoff
  - Replay: re-send a batch if a provider had an outage

## Push Notification Workers

**iOS (APNs):**
  - HTTP/2 connection to APNs; send device token + payload
  - Max payload: 4 KB; keep notifications small
  - Handle 400/410 responses → remove invalid device tokens

**Android (FCM):**
  - HTTP v1 API; send registration token + message
  - Similar token lifecycle management as APNs
  - Supports data messages (silent wake) and notification messages

**Token management:**
  - Device registers token on app install/update → stored in DB
  - Tokens expire or are invalidated on reinstall — APNs/FCM tell you
  - Cron job: prune tokens that haven't been refreshed in 90+ days

## Fan-out for Broadcast Campaigns

Sending to 100M users at once:
1. Admin creates campaign: template + target segment + schedule
2. Campaign Service queries user segment (e.g., "all US users")
3. Produces user_ids in batches to a campaign Kafka topic
4. Fan-out Workers read batches, apply preference checks, write to channel queues
5. Channel workers deliver as normal

Rate limiting on the way out:
  - APNs: no strict rate limit, but respect their throughput
  - Email: SendGrid/SES have per-second send limits — use their batch APIs
  - SMS: most expensive channel — additional confirmation step for bulk

## Delivery Tracking

Track each notification through its lifecycle:

  notification_log
  ┌──────────────────┬──────────────────────────────────┐
  │ notification_id  │ UUID        PRIMARY KEY           │
  │ user_id          │ UUID                              │
  │ type             │ VARCHAR                           │
  │ channel          │ ENUM(push, email, sms)            │
  │ status           │ ENUM(queued, sent, delivered,     │
  │                  │      failed, opened, clicked)     │
  │ provider_id      │ VARCHAR     (provider's message ID│
  │ created_at       │ TIMESTAMP                         │
  │ updated_at       │ TIMESTAMP                         │
  └──────────────────┴──────────────────────────────────┘

Delivery confirmations:
  - APNs/FCM: callback or polling for delivery receipts
  - Email: webhook from SendGrid for opens/clicks (requires tracking pixel/links)
  - SMS: Twilio webhooks for delivery status

## Deep Dives

### Deduplication
  - Events can arrive more than once (at-least-once delivery from upstream services)
  - Generate a deterministic notification_id from (user_id + event_type + event_id)
  - Check dedup cache (Redis) before enqueuing — skip if already processed
  - Window: 24h dedup to prevent double-notifications from retried events

### Priority Queues
  - Transactional notifications (order shipped, password reset): high priority, low latency
  - Marketing notifications: low priority, bulk delivery acceptable
  - Separate Kafka topics or priority levels per channel

### Template Engine
  - Templates stored in DB with variable placeholders: "Hi {{first_name}}, your order..."
  - Render at worker time (not enqueue time) — most up-to-date user data
  - Localization: store template per locale; select based on user.locale

### Rate Limiting per User
  - Protect users from notification spam during system incidents
  - Max N notifications per user per hour (configurable per type)
  - Use Redis token bucket per user_id

## Key Decisions to Highlight

1. Separate Kafka topics per channel — independent scaling and failure isolation
2. Preference check before queuing — don't enqueue what won't be delivered
3. Idempotent notification IDs — prevent duplicate delivery from upstream retries
4. Delivery status tracking — enables analytics and debugging failed deliveries
