---
title: "gRPC vs REST vs GraphQL"
tags: [grpc, rest, graphql, api-design, protobuf, http2, protocols]
difficulty: medium
estimated_time: 20min
---

## Overview

**REST (Representational State Transfer)** is the dominant API style for public-facing services. It uses HTTP verbs (`GET`, `POST`, `PUT`, `DELETE`) and URLs to represent resources. Responses are typically JSON. REST is stateless, cacheable, and widely understood — every language, framework, and tool supports it. Its weakness is **over-fetching** (getting more fields than needed) and **under-fetching** (needing multiple requests to assemble a view), and the lack of a strict contract makes client/server drift common without tooling.

**gRPC** (Google Remote Procedure Call) uses **Protocol Buffers** (`protobuf`) for schema definition and binary serialization, running over **HTTP/2**. You define services and messages in a `.proto` file; the compiler generates strongly-typed client and server stubs in any supported language. The result: significantly smaller payloads (binary vs JSON), multiplexed streams, bidirectional streaming support, and compile-time API contracts. gRPC is the standard for internal microservice communication at scale. Its weakness: no native browser support (requires grpc-web proxy), harder to debug (binary protocol), and tooling is heavier than REST.

**GraphQL** (Facebook, 2015) lets clients specify exactly which fields they want in a single query, solving REST's over/under-fetching problem. A single `/graphql` endpoint handles all queries and mutations. The schema is strongly typed and self-documenting via introspection. GraphQL is ideal for product APIs consumed by multiple clients (web, mobile, third-party) with varying data needs. Its weaknesses: query complexity can be unbounded (N+1 query problem, deeply nested queries), caching is harder (POST requests are not cached by CDNs by default), and schema evolution requires careful management.

```
REST vs gRPC vs GraphQL Comparison:

Feature           REST              gRPC              GraphQL
-----------       ----              ----              -------
Protocol          HTTP/1.1 or 2     HTTP/2 only       HTTP/1.1 or 2
Payload           JSON (text)       Protobuf (binary) JSON (text)
Schema/Contract   Optional (OAS)    Required (.proto) Required (SDL)
Type Safety       Loose             Strong (compiled) Strong (runtime)
Browser Support   Native            grpc-web proxy    Native
Streaming         No (SSE/WS side)  Bidirectional     Subscriptions
Caching           HTTP cache        Limited           Complex (POST)
Debugging         curl, Postman     grpcurl, Evans    GraphiQL, Apollo
Best For          Public APIs       Internal services Flexible clients
Over-fetching     Yes               Yes               No
Code Gen          Optional          Required          Optional

gRPC Streaming Modes:
  Unary:              Client req -> Server resp  (like REST)
  Server streaming:   Client req -> Server stream (like SSE)
  Client streaming:   Client stream -> Server resp
  Bidirectional:      Client stream <-> Server stream (like WebSocket)

REST Resource Example:
  GET  /users/123          -> {id, name, email, address, orders...}
  GET  /users/123/orders   -> [{id, items, total...}, ...]
  (2 requests; may over-fetch fields you don't need)

GraphQL Equivalent:
  POST /graphql
  { user(id: "123") { name orders { id total } } }
  -> exactly {name, orders: [{id, total}]}
  (1 request; exactly the fields requested)
```

## When to Use

- **REST**: Public APIs, third-party integrations, any API consumed by unknown clients. Default choice when interoperability and simplicity matter. Well-understood HTTP caching applies directly.
- **gRPC**: Internal service-to-service communication, polyglot microservices (one `.proto` defines the contract for all languages), high-throughput services where protobuf's smaller payload size matters, or where bidirectional streaming is needed.
- **GraphQL**: Product APIs serving multiple client types (iOS app, Android app, web app) with different data shapes. BFF (Backend for Frontend) pattern where a GraphQL layer aggregates multiple downstream REST/gRPC services. Also excellent for rapid product iteration where the data graph changes frequently.
- **When to combine**: Use gRPC internally between services for efficiency; expose a REST or GraphQL API externally for consumers. This is standard practice — gRPC for the data plane, REST/GraphQL for the API plane.

## Trade-offs & Gotchas

- **N+1 problem in GraphQL**: Naively resolving a list of users, each with their orders, triggers one DB query per user. Solution: **DataLoader** pattern (batch and deduplicate DB calls within a single request). Critical to implement before exposing GraphQL to production traffic.
- **Query depth and complexity limits**: A malicious or careless GraphQL query can request deeply nested data and exhaust your database. Enforce depth limits (max 10 levels) and complexity scoring (each field costs points, reject queries over a budget).
- **Protobuf schema evolution**: Fields are identified by number, not name, enabling backward compatibility. Never reuse field numbers. Use `reserved` for deleted fields. New fields must be optional to remain forward-compatible with old clients.
- **gRPC error model** is richer than HTTP status codes: 16 canonical status codes (`OK`, `NOT_FOUND`, `UNAVAILABLE`, `DEADLINE_EXCEEDED`, etc.). Map these correctly — don't return `UNKNOWN` for everything.
- **REST versioning**: URL versioning (`/v1/`, `/v2/`) is most visible and cache-friendly. Header versioning (`Accept: application/vnd.api+json;version=2`) is more RESTful but harder to route. Avoid versioning the entire API; prefer additive changes and deprecate fields explicitly.
- **GraphQL and caching**: Since queries are typically `POST`, HTTP caching doesn't apply. Use persisted queries (hash the query, send the hash) to enable GET-based caching. Apollo Client and Relay handle client-side caching via normalized stores.
- **gRPC in the browser**: Browsers cannot speak gRPC directly (HTTP/2 trailers not accessible via Fetch API). Use **gRPC-web** (transcoding proxy, e.g., Envoy) or **Connect** (Buf's protocol that works natively in browsers).

## Key Points for Interviews

- REST = resource-based, JSON, HTTP verbs, widely supported, cacheable, no strict contract by default.
- gRPC = protobuf schema, HTTP/2, binary payload, strongly typed, code generation, bidirectional streaming. Best for internal services.
- GraphQL = client-specified queries, single endpoint, no over/under-fetching, strong schema. Best for product APIs with diverse clients.
- Always mention N+1 + DataLoader when GraphQL is discussed — it's the most common production pitfall.
- gRPC protobuf is ~5-10x smaller and faster to parse than equivalent JSON — significant at high RPC rates.
- Combination pattern: gRPC internal + REST/GraphQL external is standard in FAANG-scale systems.
- gRPC streaming replaces WebSockets for bidirectional communication in service-to-service scenarios.
