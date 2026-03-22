---
title: "HTTP/1.1 vs HTTP/2 vs HTTP/3"
tags: [http, http2, http3, quic, performance, web-protocols]
difficulty: medium
estimated_time: 20min
---

## Overview

**HTTP/1.1** (1997) is the baseline. Each request/response pair uses a separate TCP connection by default, though `Connection: keep-alive` allows reuse. Browsers open 6-8 parallel connections per origin to work around the single-request-per-connection limitation. This is wasteful (multiple TCP handshakes, multiple TLS handshakes) and still suffers from head-of-line blocking within each connection.

**HTTP/2** (2015) multiplexes multiple request/response streams over a single TCP connection using binary framing, eliminating the need for multiple connections and enabling server push. It also compresses headers with `HPACK`, reducing overhead for repetitive headers (cookies, `User-Agent`). HTTP/2 is the current standard for browser-to-server communication and is widely supported. However, it inherits TCP's head-of-line blocking problem: a single lost TCP segment stalls all multiplexed streams until retransmission completes.

**HTTP/3** (2022, RFC 9114) replaces TCP with **QUIC** (built on UDP). QUIC implements per-stream reliability, so a lost packet only stalls the stream it belongs to, not all streams. QUIC also combines the transport handshake and TLS 1.3 negotiation, achieving **0-RTT** connection establishment for resuming sessions and **1-RTT** for new connections (vs TCP+TLS 1.3's 2-RTT minimum). HTTP/3 is now supported by all major browsers and CDNs.

```
Connection Model Comparison:

HTTP/1.1 (6 parallel connections per origin):
  Conn1: [GET /]          -> response
  Conn2: [GET /style.css] -> response
  Conn3: [GET /app.js]    -> response
  ... (head-of-line blocking within each conn)

HTTP/2 (1 connection, multiplexed streams):
  Single TCP conn:
    Stream 1: [GET /]           \
    Stream 3: [GET /style.css]   > interleaved frames
    Stream 5: [GET /app.js]     /
  (BUT: lost TCP packet stalls ALL streams)

HTTP/3 (1 QUIC connection, independent streams):
  Single QUIC/UDP conn:
    Stream 1: [GET /]           \
    Stream 2: [GET /style.css]   > lost packet only stalls its own stream
    Stream 3: [GET /app.js]     /

Handshake Latency:
  HTTP/1.1 + TLS 1.2: TCP(1.5 RTT) + TLS(2 RTT) = 3.5 RTT
  HTTP/2  + TLS 1.3:  TCP(1.5 RTT) + TLS(1 RTT) = 2.5 RTT
  HTTP/3  + QUIC:     QUIC+TLS combined = 1 RTT (0-RTT on resume)
```

## When to Use

- **HTTP/1.1**: Legacy systems, simple internal tools, or any environment where HTTP/2 support is uncertain. Still perfectly valid for server-to-server communication with connection pooling.
- **HTTP/2**: The default choice for public-facing APIs and web applications. Nginx, Envoy, and all major cloud load balancers support it. Always enable HTTP/2 between clients and your edge.
- **HTTP/3**: High-priority for latency-sensitive applications, mobile clients (frequent network switching), or any service where packet loss is common (cellular, satellite). Enable at CDN/edge layer first — CloudFront, Cloudflare, and Fastly all support HTTP/3.
- **gRPC**: Uses HTTP/2 for transport — you get multiplexing and binary framing for free. HTTP/3 support in gRPC is still maturing.

## Trade-offs & Gotchas

- HTTP/2 server push was theoretically compelling but largely deprecated in practice — browsers removed support for it. Preload hints (`<link rel="preload">`) accomplish the same goal without the complexity.
- HTTP/2 over high-loss networks can be *slower* than HTTP/1.1 with multiple connections, because a single lost TCP segment stalls all streams vs only one of six parallel HTTP/1.1 connections. HTTP/3 was designed specifically to fix this.
- **HPACK** (HTTP/2 header compression) maintains dynamic tables per connection — state that must be synchronized. If a connection is torn down, the table is lost. **QPACK** (HTTP/3) solves this with a non-blocking design.
- QUIC runs over UDP port 443. Some corporate firewalls block UDP 443, causing HTTP/3 to fall back to HTTP/2 (`Alt-Svc` header negotiation). Always implement graceful fallback.
- HTTP/2 requires TLS in practice (browsers enforce it). HTTP/3 always requires TLS — QUIC has encryption built in, there is no unencrypted HTTP/3.
- `HTTP/2 cleartext (h2c)` exists for non-TLS environments (internal service meshes) but is not supported by browsers.
- **Connection coalescing**: HTTP/2 clients may share a single connection across different hostnames that resolve to the same IP and share a TLS certificate (wildcard or SAN) — reduces connection count further.

## Key Points for Interviews

- HTTP/1.1 = one request per connection (with keep-alive, serialized). HTTP/2 = multiplexed streams on one TCP conn. HTTP/3 = multiplexed streams on QUIC (UDP).
- HTTP/2 solves application-level head-of-line blocking; HTTP/3 solves transport-level head-of-line blocking.
- HTTP/3 / QUIC = 0-RTT resumption, connection migration (IP change doesn't drop the session — critical for mobile).
- Always mention HTTP/3 when discussing CDNs, mobile performance, or last-mile optimization.
- HPACK (H2) / QPACK (H3) header compression is important for reducing overhead on high-request-rate APIs with repetitive headers.
- For system design: use HTTP/2 between services internally; push HTTP/3 at the edge for client-facing traffic.
