---
title: "TCP vs UDP"
tags: [tcp, udp, transport-layer, networking, reliability]
difficulty: easy
estimated_time: 15min
---

## Overview

**TCP (Transmission Control Protocol)** and **UDP (User Datagram Protocol)** are the two dominant Layer 4 transport protocols. TCP provides a reliable, ordered, connection-oriented byte stream: it guarantees delivery via acknowledgments (`ACK`), retransmits lost segments, and uses flow control and congestion control to avoid overwhelming receivers or the network. The cost is connection setup overhead (3-way handshake) and head-of-line blocking — a lost packet stalls all subsequent data on that connection until retransmission succeeds.

**UDP** sends self-contained datagrams with no connection setup, no delivery guarantee, no ordering, and no congestion control. What you get in return is minimal latency overhead and the freedom to implement exactly the reliability semantics your application needs — or none at all. UDP is the foundation for DNS, streaming media, VoIP, online gaming, and QUIC (HTTP/3).

The right choice depends on whether **correctness** (every byte must arrive in order) or **timeliness** (the latest data matters more than past data) dominates your use case. A video call that stutters briefly is better than one that freezes waiting for a retransmit; a bank transfer that delivers data out of order is unacceptable.

```
TCP 3-Way Handshake:
  Client                    Server
    |  ------ SYN ------->   |
    |  <----- SYN-ACK ----   |
    |  ------ ACK ------->   |
    |  (connection open)      |
    |  ------ DATA ------>   |
    |  <----- ACK --------   |
    |  (reliable delivery)    |

UDP Datagram (no handshake):
  Client                    Server
    |  ------ DATA ------->  |   (may be lost)
    |  ------ DATA ------->  |   (may arrive out of order)
    |  (no ACK, no guarantee) |

TCP Connection Teardown (4-way):
  Client --> FIN --> Server
  Client <-- ACK <-- Server
  Client <-- FIN <-- Server
  Client --> ACK --> Server
  (TIME_WAIT state: 2*MSL before port reuse)
```

## When to Use

- **TCP**: Any protocol where correctness is paramount — HTTP/1.1, HTTP/2, database connections (`postgres://`, MySQL), SSH, SMTP, file transfers. If you can't tolerate data loss or reordering, use TCP.
- **UDP**: Latency-sensitive, real-time applications where a stale retransmit is worse than a gap — video/audio streaming, VoIP, online gaming (position updates), DNS queries, DHCP.
- **UDP with application-layer reliability**: QUIC (HTTP/3) uses UDP but implements its own packet-level reliability and congestion control, avoiding head-of-line blocking at the transport layer while still guaranteeing delivery at the stream level.
- **Multicast/broadcast**: Only possible with UDP — used for service discovery (mDNS, SSDP) and live video distribution to many receivers simultaneously.

## Trade-offs & Gotchas

- TCP head-of-line blocking: a single lost packet blocks all data behind it on the same connection. HTTP/2 multiplexes streams over one TCP connection, which worsens this problem under packet loss. HTTP/3 (QUIC) solves it by using UDP with per-stream reliability.
- TCP `TIME_WAIT` state (2*MSL, typically 60-120s) can exhaust ephemeral ports under high connection churn. Solutions: `SO_REUSEADDR`, `SO_REUSEPORT`, connection pooling, or `tcp_tw_reuse` sysctl.
- **Nagle's algorithm** batches small TCP writes to reduce packet count — can cause latency spikes for interactive protocols. Disable with `TCP_NODELAY` (used by Redis, game servers, SSH).
- UDP has no congestion control, so misbehaving UDP applications can saturate a network link and starve TCP flows. QUIC addresses this by implementing congestion control in user space.
- Firewalls and NAT devices handle TCP state machines natively; UDP flows must be tracked heuristically. Long-idle UDP connections through NAT may have their mappings silently dropped — applications send keepalive packets to avoid this.
- TCP buffer tuning: `SO_SNDBUF` and `SO_RCVBUF` — the OS auto-tunes these in Linux, but high-throughput systems (e.g., file storage, inter-DC replication) often benefit from explicit tuning.

## Key Points for Interviews

- TCP = reliable, ordered, connection-oriented. UDP = unreliable, unordered, connectionless.
- TCP 3-way handshake adds 1.5 round trips before data flows — matters for latency-sensitive short connections (reason HTTP/2 uses persistent connections, HTTP/3 uses 0-RTT).
- Head-of-line blocking is a TCP weakness exploited in system design discussions about HTTP/2 vs HTTP/3.
- QUIC = UDP + reliability + encryption + multiplexing — the modern answer to TCP's limitations.
- Use UDP when your application can tolerate loss better than it can tolerate delay (gaming, video, DNS).
- `TCP_NODELAY` disables Nagle's algorithm — mention this when designing latency-sensitive protocols.
- Connection pooling (databases, HTTP keep-alive) amortizes TCP handshake cost — always mention this when designing high-throughput services.
