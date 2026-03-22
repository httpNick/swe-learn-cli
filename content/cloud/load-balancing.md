---
title: "Load Balancing"
tags: [load-balancing, alb, nlb, l4, l7, health-checks, routing-algorithms]
difficulty: medium
estimated_time: 20min
---

## Overview

A load balancer distributes incoming traffic across a pool of backend servers, providing
horizontal scalability and high availability. Without a load balancer, a single server becomes
both a throughput bottleneck and a single point of failure. Every scalable system design places
a load balancer in front of stateless service tiers.

Load balancers operate at different OSI layers. Layer 4 (transport) load balancers route based
on IP address and TCP/UDP port — they're fast and protocol-agnostic but cannot inspect HTTP
content. Layer 7 (application) load balancers terminate HTTP/HTTPS, read headers, paths, and
cookies, and can make routing decisions based on that content. L7 is slower than L4 due to
the additional parsing but far more flexible.

On AWS, the Application Load Balancer (ALB) operates at L7 and is the standard choice for
HTTP/HTTPS workloads. It supports path-based routing (/api/* → API servers, /static/* → S3),
host-based routing (api.example.com vs static.example.com), WebSocket, and HTTP/2. The Network
Load Balancer (NLB) operates at L4, handles millions of requests per second with ultra-low
latency, supports static Elastic IPs, and integrates with AWS PrivateLink.

## When to Use

- **ALB**: HTTP/HTTPS microservices, REST APIs, WebSocket connections, routing to multiple
  target groups based on path or hostname.
- **NLB**: TCP/UDP workloads, extreme throughput requirements, static IP requirements,
  gaming servers, IoT, or any non-HTTP protocol.
- **Global load balancing**: multi-region deployments — route users to the nearest region
  using latency-based DNS (Route 53) or Anycast (AWS Global Accelerator).
- **Internal LB**: service-to-service communication within a VPC — expose services privately
  without going through the internet.

## Trade-offs & Gotchas

- ALB adds ~1ms of latency vs NLB ~0.1ms — negligible for most applications, significant for
  high-frequency trading or real-time gaming.
- Sticky sessions (session affinity) bind a user to a specific backend instance, breaking true
  statelessness. Prefer stateless backends with shared session storage (Redis) instead.
- Health check tuning matters: too aggressive (short intervals, low thresholds) causes flapping;
  too lenient means traffic routes to unhealthy instances longer than necessary.
- Connection draining (deregistration delay): give in-flight requests time to complete before
  removing an instance from the pool (default 300s on AWS).
- ALB does not support static IPs — use NLB or Global Accelerator if you need to whitelist IPs
  (e.g., for firewall rules at enterprise customers).
- Weighted routing enables canary deploys: send 5% of traffic to a new target group and
  gradually increase as confidence grows.

## Architecture Diagram

```
  L7 (ALB) Path-Based Routing:
  [Internet]
      |
  [ALB :443]
   /         \
[/api/*]   [/static/*]
    |            |
[API Servers] [S3 / Lambda]

  L4 (NLB) TCP Pass-Through:
  [Client] --> [NLB] --> [Backend Pool]
  (TLS terminated at backend, static IP on NLB)

  Global Load Balancing:
  [User: US-East] --> [Route 53 latency] --> [us-east-1 ALB]
  [User: EU]      --> [Route 53 latency] --> [eu-west-1 ALB]
                            |
                    (failover: if health check fails,
                     route to secondary region)

  Routing Algorithms:
  Round Robin       -- equal distribution, simple
  Least Connections -- better for variable request duration
  IP Hash           -- sticky by client IP (session affinity without cookies)
  Weighted          -- canary deploys, A/B testing
```

## Key Interview Points

- L4 = IP/port routing, fast, protocol-agnostic. L7 = HTTP-aware, content-based routing, more flexible.
- ALB is the default for HTTP workloads; NLB for TCP/UDP, static IPs, or extreme throughput.
- Always mention health checks — a load balancer is only as good as its ability to detect and
  remove unhealthy backends.
- Avoid sticky sessions; prefer stateless backends with a shared session store (Redis/ElastiCache).
- Connection draining prevents dropped in-flight requests during deployments or scale-in events.
- For global systems, combine regional ALBs with Route 53 latency routing or Global Accelerator
  for Anycast edge termination.
