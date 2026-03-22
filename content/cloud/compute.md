---
title: "Cloud Compute"
tags: [compute, vms, containers, serverless, auto-scaling, kubernetes]
difficulty: easy
estimated_time: 20min
---

## Overview

Cloud compute offers three primary deployment models: virtual machines (VMs), containers, and
serverless functions. Each trades off control, operational overhead, cold-start latency, and cost
differently. Picking the right model in a system design interview signals architectural maturity.

Virtual machines provide full OS-level isolation with the most control — you choose the CPU,
memory, OS, and runtime. They are slow to provision (minutes), carry the highest operational
burden (patching, AMI management), but have zero cold-start latency once running and support
any workload. Use VMs for stateful services, databases, or specialized hardware (GPU instances).

Containers (Docker + Kubernetes) share the host OS kernel, start in seconds, and pack more
workloads per host than VMs. Kubernetes adds orchestration: scheduling, health checks, rolling
deploys, and horizontal scaling. The tradeoff is operational complexity — running a Kubernetes
cluster requires significant investment. Managed services (EKS, GKE, AKS) reduce but don't
eliminate that burden.

Serverless functions (Lambda, Cloud Functions) abstract away all infrastructure. You deploy code
and pay per invocation. Scaling is automatic and instant — but cold starts (100ms–2s) affect
latency-sensitive paths, execution is time-limited (15min for Lambda), and local state is
ephemeral. Best suited for event-driven, bursty, or infrequent workloads.

## When to Use

- **VMs**: long-running stateful services, GPU workloads, legacy apps, anything needing OS-level
  access or specialized hardware.
- **Containers (K8s)**: microservices, APIs, batch jobs — when you need portability and want
  consistent environments from dev to prod.
- **Serverless**: event handlers, webhook processors, scheduled jobs, glue code between services,
  or spiky workloads with low baseline traffic.
- **Auto-scaling**: always mention it for stateless services that see variable traffic. State the
  metric you'd scale on (CPU, request count, queue depth).

## Trade-offs & Gotchas

- Serverless cold starts hurt p99 latency — mitigate with provisioned concurrency (Lambda) or
  by keeping functions warm with scheduled pings. Not suitable for hard latency SLAs.
- Kubernetes is powerful but operationally heavy. Don't propose self-managed K8s unless you
  have a reason; prefer managed (EKS/GKE).
- Auto-scaling has a cooldown period (default 300s for AWS ASG) — size your baseline capacity
  to absorb traffic spikes during scale-out lag.
- Vertical scaling (bigger instance) has an upper bound and requires a restart; horizontal scaling
  (more instances) is preferred for stateless services.
- Containers don't solve all portability problems — OS-specific dependencies and stateful storage
  remain tricky.
- Scale-in (terminating instances) can cause dropped connections — use connection draining and
  graceful shutdown hooks.

## Architecture Diagram

```
  VM-based:
  [Load Balancer] --> [EC2 Instance 1]
                  --> [EC2 Instance 2]   (Auto Scaling Group)
                  --> [EC2 Instance N]

  Container-based (Kubernetes):
  [Load Balancer]
       |
  [Ingress Controller]
       |
  [Service]
   /       \
[Pod 1]  [Pod 2]   (Deployment, HPA scales pod count)
   |         |
  [Node 1] [Node 2] (EC2 instances / managed node group)

  Serverless:
  [API Gateway] --> [Lambda Function] --> [DynamoDB]
  [S3 Event]    --> [Lambda Function] --> [SQS]
  [CloudWatch]  --> [Lambda (cron)]   --> [RDS]

  Auto-scaling triggers:
  CPU > 70%  --> scale out (+2 instances)
  CPU < 30%  --> scale in  (-1 instance, with cooldown)
  Queue depth > 1000 --> scale out workers
```

## Key Interview Points

- Lead with containers/K8s for microservice APIs; serverless for event-driven or spiky workloads;
  VMs only when you need OS control or stateful storage.
- Always pair stateless horizontal scaling with a load balancer — make this explicit.
- Serverless cold starts are a real concern — mention provisioned concurrency for latency-critical
  paths or use containers with K8s instead.
- Auto-scaling metrics: CPU is a proxy; prefer request-rate or queue-depth for more direct scaling
  signals.
- Spot/Preemptible instances cut costs 60-90% for fault-tolerant batch workloads — worth
  mentioning for data processing pipelines.
- Cooldown periods and connection draining prevent scale-in from causing errors.
