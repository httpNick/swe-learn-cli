---
title: "Reverse Proxies and API Gateways"
tags: [reverse-proxy, api-gateway, nginx, envoy, kong, apigee, aws-api-gateway, service-mesh]
difficulty: medium
estimated_time: 20min
---

## Overview

A **reverse proxy** sits in front of backend servers, accepting client requests on their behalf. Unlike a forward proxy (which sits in front of clients and proxies their outbound traffic), a reverse proxy is transparent to the client — the client thinks it's talking directly to the service. Reverse proxies provide TLS termination, load balancing, request routing, compression, caching, and a choke point for cross-cutting concerns. **Nginx** and **Envoy** are the dominant open-source reverse proxies.

An **API Gateway** is a reverse proxy specialized for API management. Beyond basic routing, it adds: **authentication and authorization** (API keys, OAuth 2.0 / JWT validation), **rate limiting** (per-client, per-endpoint), **request/response transformation** (header manipulation, payload translation), **API versioning**, **analytics and billing metering**, and **developer portal** features. Managed options include **AWS API Gateway**, **Google Apigee**, and **Kong** (open-source with an enterprise tier). API Gateways are the standard entry point for external API consumers.

A **service mesh** (Istio, Linkerd) extends the reverse proxy pattern to service-to-service communication inside a cluster. Each service gets a sidecar proxy (Envoy) that handles mTLS, load balancing, circuit breaking, retries, and observability — without any application code changes. This separates infrastructure concerns (networking policy) from business logic, enabling consistent behavior across a polyglot microservice fleet.

```
Reverse Proxy Topology:

  [Internet]
      |
  [Reverse Proxy: Nginx / Envoy / ALB]
      |          |            |
  [Service A] [Service B] [Service C]

  Proxy responsibilities:
  - TLS termination (presents cert to client)
  - Routing (path/host-based)
  - Load balancing across service instances
  - Compression (gzip, brotli)
  - Request buffering (protect slow backends)
  - Access logging

API Gateway Topology:

  [External Client]
      |
  [API Gateway: Kong / Apigee / AWS API GW]
      |
  [Auth Plugin] --> validate JWT / API key
  [Rate Limit]  --> reject if over quota
  [Transform]   --> strip/add headers, map payload
      |
  [Backend Microservices]

Service Mesh Topology:

  [Service A] <--> [Envoy Sidecar A] <--> [Envoy Sidecar B] <--> [Service B]
                         |                         |
                   mTLS, retries,           mTLS, circuit break,
                   tracing, metrics         load balancing

  Control Plane (Istiod): distributes config to all sidecars
  Data Plane (Envoy): enforces it per-request
```

## When to Use

- **Nginx as reverse proxy**: TLS offload for any HTTP service, static file serving, simple path-based routing, rate limiting at the edge. Default choice for self-hosted infrastructure.
- **Envoy**: When you need dynamic configuration (xDS API), advanced load balancing algorithms, distributed tracing (Zipkin/Jaeger), or you're building a service mesh. Envoy is the data plane for Istio, AWS App Mesh, and Google Traffic Director.
- **Kong**: Self-hosted API gateway with a plugin ecosystem. Good for teams that want API gateway features without full vendor lock-in. Runs on top of Nginx (via OpenResty) or Envoy.
- **AWS API Gateway**: Fully managed, serverless-friendly (native Lambda integration), pay-per-request. Best for AWS-native architectures. Two variants: REST API (feature-rich, more expensive) and HTTP API (lower latency, cheaper, fewer features).
- **Apigee (Google)**: Enterprise API management — strong for organizations that need developer portals, API monetization, complex transformation, and multi-cloud policy enforcement. Common in financial services and telco.
- **Service mesh (Istio/Linkerd)**: When you need consistent mTLS, observability, and traffic policy across many services in Kubernetes and don't want to implement those concerns in each service. Adds operational complexity — worth it at 10+ services.

## Trade-offs & Gotchas

- **API Gateway as a bottleneck**: All traffic flows through the gateway — it must be highly available and sized for peak throughput. A misconfigured rate limit or auth plugin failure can take down your entire API. Deploy in multiple AZs, set generous timeouts, and circuit-break downstream failures.
- **Latency added by API Gateway**: Each plugin (auth, rate limiting, logging) adds latency. AWS API Gateway REST API adds ~5-10ms; HTTP API adds ~1ms. Kong with multiple plugins can add 5-20ms. Account for this in your SLA math.
- **AWS API Gateway vs ALB + custom auth**: API Gateway is convenient but expensive at high volume (pay per million requests). At sufficient scale, an ALB + Lambda authorizer or a self-hosted Kong on ECS may be cheaper. Do the math.
- **JWT validation at the gateway vs service**: Validating JWTs at the gateway (offloading from services) is standard, but the gateway must cache public keys (JWKS) to avoid hitting the identity provider on every request. Key rotation must propagate to the gateway.
- **Nginx config drift**: Nginx is configured via static files — changes require a reload (`nginx -s reload`). Envoy uses a dynamic xDS API — config changes apply in seconds without restarts. For rapidly changing routing rules, Envoy is operationally superior.
- **Service mesh overhead**: Each Envoy sidecar adds ~5-10ms per hop and ~50-100MB memory. In a call chain of 5 services, that's 25-50ms added. At low request rates it's negligible; at high concurrency it compounds. Measure before enabling mesh on latency-sensitive paths.
- **Egress control**: Reverse proxies/gateways typically handle *ingress*. Egress (outbound traffic from services to external APIs) is often uncontrolled. Service mesh sidecar proxies handle egress too — enforce egress policy to prevent data exfiltration and unintended external dependencies.

## Key Points for Interviews

- Reverse proxy = routing, TLS termination, load balancing, compression. Sits in front of backends.
- API Gateway = reverse proxy + auth, rate limiting, transformation, analytics. Entry point for external APIs.
- Kong = self-hosted, plugin-based, open-source. AWS API Gateway = managed, pay-per-request, Lambda-native. Apigee = enterprise, monetization, complex policy.
- Service mesh (Istio + Envoy sidecars) handles east-west (service-to-service) traffic; API gateway handles north-south (external-to-service).
- Envoy's xDS dynamic config is a major operational advantage over static Nginx config for frequently changing routing rules.
- JWT validation at the gateway offloads auth from every service — cache JWKS to avoid latency on every request.
- AWS API Gateway REST vs HTTP API: REST = more features, higher cost/latency. HTTP API = lower latency (~1ms added), cheaper, use it unless you need REST-specific features (caching, API key usage plans).
