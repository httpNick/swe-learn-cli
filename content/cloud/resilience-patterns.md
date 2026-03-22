---
title: "Resilience Patterns"
tags: [resilience, circuit-breaker, bulkhead, retry, backoff, timeouts, backpressure, idempotency]
difficulty: medium
estimated_time: 25min
---

## Overview

Resilience patterns prevent failures in one service from cascading into system-wide outages.
In a distributed system with 10 services each at 99.9% availability, the probability of all
being healthy simultaneously is 99.9%^10 = 99.0%. Without resilience patterns, a single slow
or failing dependency can exhaust thread pools, fill queues, and bring down healthy services.

The circuit breaker pattern wraps calls to a dependency and monitors for failures. When the failure
rate exceeds a threshold, the circuit "opens" — subsequent calls fail immediately without
attempting the dependency. After a configurable reset timeout, it enters "half-open" state,
allowing a probe request through. If that succeeds, the circuit closes; if it fails, it reopens.
This prevents cascading failures by fast-failing rather than letting threads pile up waiting for
a hanging dependency.

The bulkhead pattern isolates resources per dependency. Named after ship compartments that prevent
flooding from sinking the whole vessel. In practice: separate thread pools or connection pools
per downstream service. If Service B becomes slow and exhausts its dedicated thread pool, Service
A's thread pool (used for calling Service C) is unaffected. Without bulkheads, a slow dependency
monopolizes the shared thread pool and starves all other calls.

Retries are necessary for transient failures (network blips, temporary overload) but dangerous
without exponential backoff and jitter. Naive fixed-interval retries create thundering herds —
all clients retry simultaneously, amplifying load on an already-struggling service. Exponential
backoff doubles the wait time each retry (1s, 2s, 4s, 8s). Jitter adds randomness to spread
retries across the window, preventing synchronized retry storms.

## When to Use

- **Circuit breaker**: any synchronous call to an external service or dependency that could
  degrade or fail. Especially important for non-critical downstream dependencies.
- **Bulkhead**: when a service calls multiple dependencies — isolate each to prevent one slow
  dependency from starving others.
- **Retry + backoff**: transient network errors, rate limit responses (429), temporary service
  unavailability. Do NOT retry on 4xx client errors (except 429) — they will not succeed.
- **Timeout**: every network call, database query, and external API call — no exceptions.
- **Backpressure**: producers that can outpace consumers — bound your queues and reject or
  throttle when the queue is full.

## Trade-offs & Gotchas

- Circuit breaker threshold tuning: too sensitive = flapping on minor blips; too lenient = slow
  to react to real outages. Track error rates over a sliding window, not a fixed count.
- Retry amplification: with N services retrying to M services, failures can amplify traffic by
  N*max_retries. Combine retries with circuit breakers to limit amplification.
- Idempotency is required for safe retries: if a retry re-executes a payment or sends a duplicate
  notification, you have a bug. Use idempotency keys (UUID per logical operation, stored in DB)
  to deduplicate.
- Timeouts must be calibrated: too short = false failures during high load; too long = threads
  held for the full duration. Use p99 latency * 2 as a starting point, then tune.
- Deadline propagation: set a total budget for the request (e.g., 500ms) and propagate it to
  downstream calls so they don't waste time on a request whose parent already timed out.
- Backpressure implementations: TCP flow control, bounded blocking queues, HTTP 429 responses,
  gRPC flow control — all are forms of the same pattern.

## Architecture Diagram

```
  Circuit Breaker States:
  [CLOSED] -- failure rate < threshold --> normal operation
      |
  failure rate exceeds threshold (e.g., >50% in 10s window)
      |
  [OPEN] -- fast-fail all requests, return fallback/error immediately
      |
  reset timeout expires (e.g., 30s)
      |
  [HALF-OPEN] -- allow 1 probe request
      |            |
   success      failure
      |            |
  [CLOSED]     [OPEN]

  Bulkhead (Thread Pool Isolation):
  [Service A]
   |           |
  Pool-B     Pool-C     (separate pools per dependency)
  (10 threads) (10 threads)
   |           |
  [Svc B]    [Svc C]
  (slow)     (healthy)
  Pool-B     Pool-C unaffected -- Service A still serves Pool-C traffic

  Retry with Exponential Backoff + Jitter:
  attempt 1: immediate
  attempt 2: wait 1s + jitter(0-0.5s)
  attempt 3: wait 2s + jitter(0-1s)
  attempt 4: wait 4s + jitter(0-2s)
  give up after 3 retries or circuit opens

  Idempotency Key Flow:
  [Client] sends request with Idempotency-Key: uuid-123
  [Server] checks key in idempotency table
    found: return cached response (no re-execution)
    not found: execute, store result with key, return
```

## Key Interview Points

- Circuit breaker prevents cascading failures by fast-failing when a dependency is degraded.
  Mention the three states: closed, open, half-open.
- Bulkhead limits the blast radius of a slow dependency by isolating its resource allocation.
- Always use exponential backoff + jitter for retries — never fixed-interval, which causes
  thundering herd.
- Every network call needs a timeout — a hanging call without a timeout will hold a thread
  indefinitely and starve the thread pool.
- Retries require idempotency — design operations with idempotency keys to make retries safe.
- Backpressure: bound your queues and reject/throttle excess load rather than accepting
  unbounded work that causes unbounded latency.
