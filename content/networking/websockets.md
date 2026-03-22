---
title: "WebSockets and Long Polling"
tags: [websockets, long-polling, sse, real-time, push-notifications, bidirectional]
difficulty: medium
estimated_time: 15min
---

## Overview

Standard HTTP is a request-response protocol — the client asks, the server answers, connection closes (or is kept alive for reuse). For **real-time bidirectional communication** (chat, collaborative editing, live dashboards, gaming), HTTP's pull model is inefficient. Three progressively better approaches exist: **short polling**, **long polling**, and **WebSockets**.

**Long polling** simulates push by having the client send a request that the server holds open until it has data (or a timeout). On data arrival or timeout, the server responds, the client immediately sends a new long-poll request. It works over plain HTTP and passes through any proxy, but is inefficient: each message requires a new HTTP request/response cycle with full headers, and the server must hold many open connections, each consuming a thread or file descriptor.

**WebSockets** (RFC 6455) upgrade an existing HTTP/1.1 connection via the `Upgrade: websocket` header, then switch to a persistent, full-duplex, framed binary/text protocol over the same TCP connection. Either side can send messages at any time with no per-message HTTP overhead. This makes WebSockets ideal for high-frequency, bidirectional communication. **Server-Sent Events (SSE)** occupy the middle ground: a persistent HTTP connection with one-way server-to-client push (text/event-stream), simpler than WebSockets but sufficient for read-only feeds (live scores, notifications).

```
Short Polling (inefficient, simple):
  Client --> GET /messages?since=t1 --> Server (returns immediately, empty if no data)
  Client --> GET /messages?since=t1 --> Server  (repeat every N seconds)
  Cost: wasted requests, high latency proportional to poll interval

Long Polling:
  Client --> GET /messages?since=t1 --> Server
                                    (server holds connection open...)
                                    (... new message arrives ...)
                          <-- 200 {message} ---
  Client --> GET /messages?since=t2 --> Server (immediately re-requests)
  Better: ~0 latency for delivery. Worse: HTTP overhead per message.

WebSocket Upgrade Handshake:
  Client --> HTTP GET /chat
             Upgrade: websocket
             Connection: Upgrade
             Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
             Sec-WebSocket-Version: 13
  Server <-- HTTP 101 Switching Protocols
             Upgrade: websocket
             Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=

  After upgrade, TCP connection carries WebSocket frames:
  Client <--> Server  (full-duplex, any time, minimal overhead)

SSE (Server-Sent Events) - server push only:
  Client --> GET /live-feed
             Accept: text/event-stream
  Server --> HTTP 200
             Content-Type: text/event-stream
             (connection stays open)
             data: {"score": "3-2"}\n\n
             data: {"score": "4-2"}\n\n
```

## When to Use

- **WebSockets**: Bidirectional, high-frequency messaging — chat applications, collaborative document editing (Google Docs style), multiplayer games, live trading dashboards, real-time collaborative features.
- **SSE (Server-Sent Events)**: One-way server push, browser clients, where reconnection and event IDs are needed — live news feeds, notification streams, build/CI log tailing, live score updates. SSE reconnects automatically and has built-in event IDs for resumption.
- **Long Polling**: When WebSockets are unavailable (strict proxies, old infrastructure) or for infrequent events where the simplicity of HTTP is worth the overhead. Also used as a fallback in libraries like Socket.IO.
- **Short Polling**: Only for very infrequent updates (once per minute) where implementation simplicity matters more than efficiency. Avoid for anything approaching real-time.
- **When NOT to use WebSockets**: Simple request/response patterns, cacheable data, or communication through intermediaries that don't support WebSocket upgrades (some corporate proxies, old CDNs). Check that your CDN/load balancer supports `Upgrade` headers.

## Trade-offs & Gotchas

- WebSocket connections are **stateful and sticky** — they land on one backend server for their lifetime. Load balancers must support WebSocket forwarding. This complicates horizontal scaling: a server restart drops all connected clients. Mitigate with a **message broker** (Redis Pub/Sub, Kafka) so any backend can handle any client's messages.
- **Connection limits**: A server holding 100k WebSocket connections needs careful file descriptor tuning (`ulimit -n`, `fs.file-max`) and memory management. Each connection holds buffers. Nginx + Node.js can handle ~10k-50k concurrent WebSocket connections per process depending on message rate.
- **Heartbeats / ping-pong**: TCP keepalive is too slow (default 2 hours). WebSocket has a built-in ping/pong frame mechanism. Send application-level heartbeats every 30s to detect dead connections quickly and free resources.
- Proxies and firewalls that don't understand WebSockets will drop the `Upgrade` header or timeout idle connections. Always implement reconnection logic with exponential backoff in the client.
- **Backpressure**: Unlike HTTP/2 which has per-stream flow control, WebSocket has no built-in backpressure. A slow consumer can accumulate unbounded in-memory buffers. Implement application-level flow control if message rate can spike.
- SSE is HTTP — it works through HTTP/2 (multiplexed streams), while WebSocket requires special handling over HTTP/2. For browser clients, SSE + a separate REST endpoint for client-to-server messages can replace WebSockets cleanly.

## Key Points for Interviews

- WebSocket = full-duplex, persistent TCP connection with minimal framing overhead. Upgrade from HTTP via `101 Switching Protocols`.
- SSE = persistent HTTP, server-to-client only, auto-reconnect, text only. Simpler than WebSocket for read-only push.
- Long polling = HTTP held open until data is ready. Works everywhere, higher overhead than WebSocket.
- WebSockets are stateful — route with sticky load balancing or use a message broker (Redis Pub/Sub, Kafka) so any server can handle any client.
- Always implement client-side reconnection with exponential backoff. Connections drop.
- Heartbeats prevent ghost connections — send a ping every 30s, close on missed pong.
- For system design: separate the WebSocket connection layer (stateful) from the application logic layer (stateless), connected via a pub/sub bus.
