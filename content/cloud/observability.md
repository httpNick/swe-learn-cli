---
title: "Observability"
tags: [observability, metrics, logging, tracing, prometheus, grafana, elk, opentelemetry, slo]
difficulty: medium
estimated_time: 25min
---

## Overview

Observability is your ability to understand the internal state of a system from its external
outputs. The three pillars are metrics (aggregated numbers), logs (discrete events), and traces
(request flows across services). A system without observability is a black box — you can't debug
incidents you can't see, and you can't improve what you can't measure.

Metrics are numerical measurements sampled over time: request rate, error rate, latency
percentiles, CPU utilization, queue depth. Prometheus is the dominant open-source metrics system:
services expose a /metrics endpoint (or push via Pushgateway) and Prometheus scrapes on a
configurable interval. Metrics are stored as time-series and queried with PromQL. High cardinality
(too many unique label values) kills Prometheus performance — avoid using user IDs or request IDs
as label values.

Logs capture discrete events: a request received, an exception thrown, a state transition. The
ELK stack (Elasticsearch + Logstash + Kibana) or EFK (with Fluentd) centralizes logs from all
services. Structured logging (JSON with consistent fields: timestamp, level, service, trace_id,
message) is essential — unstructured text logs are hard to query and alert on. Always include a
trace ID in logs so you can correlate a log entry with its distributed trace.

Distributed tracing tracks a request as it flows through multiple services. Each service adds a
span (start time, duration, metadata) to the trace. Traces are assembled by a collector (Jaeger,
Zipkin, AWS X-Ray) and visualized as waterfall diagrams to pinpoint latency bottlenecks. Sampling
is necessary at high throughput — 100% trace collection is prohibitively expensive; 1–5% sampling
with head-based or tail-based strategies is typical.

## When to Use

- **Metrics + Grafana**: always. This is table stakes for any production system. Define SLIs
  (latency p99, error rate, availability) and set SLO thresholds.
- **Centralized logging**: aggregate logs from all services in one place for incident debugging
  and audit trails.
- **Distributed tracing**: microservices architectures where a single request touches 3+
  services — you need trace IDs to follow the request end-to-end.
- **Alerting**: alert on SLO burn rate (error budget consumption), not on raw metrics thresholds
  alone — reduces alert fatigue.

## Trade-offs & Gotchas

- High cardinality metrics (user_id, request_id as labels) explode Prometheus's memory and
  storage. Use traces for per-request data, metrics for aggregates.
- Log volume grows fast — set retention policies (30–90 days hot, archive to S3 for compliance).
  Index only fields you query on to control Elasticsearch cost.
- 100% trace sampling at high QPS is expensive. Use tail-based sampling (keep traces with errors
  or high latency; sample the rest) to maximize signal-to-noise.
- Alert fatigue is real: too many alerts = on-call engineers ignore them. Alert on symptoms
  (user-facing error rate) not causes (CPU spike on one instance).
- Correlation between pillars requires a shared trace_id flowing through logs, metrics labels,
  and trace spans — instrument at the framework/middleware level so it's automatic.
- OpenTelemetry (OTEL) is the CNCF-standard instrumentation API. Instrument once, export to any
  backend (Jaeger, Datadog, Honeycomb) — avoids vendor lock-in.

## Architecture Diagram

```
  Three Pillars:
  [Service A] ---> Metrics (counters, histograms) --> [Prometheus] --> [Grafana]
  [Service A] ---> Logs (JSON + trace_id)         --> [Fluent Bit]
  [Service A] ---> Traces (spans)                 --> [OTEL Collector]
                                                         |
                                              [Jaeger / Datadog / Honeycomb]

  SLO Alerting:
  SLI: error rate = errors / total_requests
  SLO: error rate < 0.1% over 30 days
  Error budget: 0.1% * 30days * 24h * 60min = ~43 minutes of downtime allowed

  Alert: burn rate > 14x (budget consumed 14x faster than allowed)
  --> Page on-call immediately

  Golden Signals (Google SRE):
  Latency    -- p50, p95, p99 response time
  Traffic    -- requests per second
  Errors     -- 4xx/5xx rate, exception rate
  Saturation -- CPU, memory, queue depth, connection pool utilization
```

## Key Interview Points

- Three pillars: metrics (what is happening), logs (what happened), traces (why it happened and
  where time was spent).
- Golden signals: latency, traffic, errors, saturation — alert on these before custom metrics.
- SLI/SLO/error budget: define measurable SLIs, set SLO targets, alert on error budget burn rate.
  This is the Google SRE model.
- Avoid high-cardinality metric labels — use traces for per-request details.
- Structured logging with trace_id correlation across all three pillars enables fast incident
  diagnosis.
- OpenTelemetry for vendor-neutral instrumentation — mention it to signal awareness of modern
  observability practices.
