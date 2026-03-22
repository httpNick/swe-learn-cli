---
title: "OSI Model"
tags: [osi, networking, layers, tcp-ip, protocols]
difficulty: easy
estimated_time: 15min
---

## Overview

The **OSI (Open Systems Interconnection) model** is a 7-layer conceptual framework that standardizes how network communication is decomposed into discrete responsibilities. Each layer serves the layer above it and is served by the layer below it. In practice, most engineers work with the TCP/IP model (4 layers), but OSI vocabulary is universal in system design discussions — especially when distinguishing L4 vs L7 load balancing, or diagnosing where a failure occurred.

The layers from bottom to top: **Physical** (bits over wire/radio), **Data Link** (frames between adjacent nodes, MAC addresses), **Network** (IP routing between hosts), **Transport** (end-to-end delivery, ports, TCP/UDP), **Session** (connection lifecycle management), **Presentation** (encoding, encryption, compression), **Application** (HTTP, DNS, SMTP — what the user's software speaks).

In real systems, the Session and Presentation layers are largely collapsed into the Application layer. When engineers say "L4" they mean Transport (TCP/UDP), and "L7" means Application (HTTP/gRPC). These two are the layers that matter most for infrastructure decisions.

```
OSI Layer Reference:
+-----+-------------+------------------+-----------------------------+
| No. | Name        | Unit             | Example Protocols           |
+-----+-------------+------------------+-----------------------------+
|  7  | Application | Message/Data     | HTTP, DNS, SMTP, FTP, gRPC  |
|  6  | Presentation| Message/Data     | TLS/SSL, JPEG, JSON encoding |
|  5  | Session     | Message/Data     | TLS session, RPC sessions   |
|  4  | Transport   | Segment/Datagram | TCP, UDP, QUIC              |
|  3  | Network     | Packet           | IP, ICMP, BGP, OSPF         |
|  2  | Data Link   | Frame            | Ethernet, Wi-Fi (802.11)    |
|  1  | Physical    | Bits             | Copper, fiber, radio waves  |
+-----+-------------+------------------+-----------------------------+

TCP/IP Model Mapping:
OSI L7+L6+L5 --> TCP/IP Application Layer
OSI L4        --> TCP/IP Transport Layer
OSI L3        --> TCP/IP Internet Layer
OSI L2+L1     --> TCP/IP Network Access Layer
```

## When to Use

- **L4 vs L7 decisions**: When discussing load balancers, firewalls, or proxies, always specify which layer they operate at. L4 = fast, protocol-agnostic; L7 = content-aware, more flexible.
- **Firewall rules**: Security groups and NACLs operate at L3/L4 (IP/port). WAFs operate at L7 (HTTP headers, body).
- **Debugging**: "Is this a network problem or an application problem?" maps directly to which OSI layer is failing — packet loss is L1/L2/L3, connection refused is L4, HTTP 500 is L7.
- **Protocol selection**: Choosing between TCP and UDP, or between REST and gRPC, is a L4 and L7 decision respectively.

## Trade-offs & Gotchas

- The OSI model is descriptive, not prescriptive — real protocols often blur layer boundaries. TLS spans L4-L6, QUIC combines L3/L4 transport with L7 concerns.
- "L3 load balancer" (IP-level) is rarely seen in cloud environments; L4 (NLB) and L7 (ALB) are the practical distinction.
- Encapsulation is the key mechanism: each layer wraps the payload from the layer above with its own header. A TCP segment is encapsulated in an IP packet, which is encapsulated in an Ethernet frame.
- **MTU (Maximum Transmission Unit)** is a Layer 2/3 concern. Ethernet MTU is 1500 bytes. TCP MSS (Maximum Segment Size) adjusts to avoid IP fragmentation — important for performance tuning and debugging `PMTUD` issues.
- Don't confuse the OSI model with the TCP/IP stack. Interviews that ask about "layers" almost always mean the TCP/IP stack in practice.

## Key Points for Interviews

- L4 (Transport) = TCP/UDP, identified by IP + port. L7 (Application) = HTTP, DNS, gRPC — content-aware.
- Load balancers: L4 routes by IP/port, L7 routes by URL path, headers, cookies.
- Firewalls/security groups operate at L3/L4; WAFs operate at L7.
- Encapsulation: each layer adds a header; each layer on the receiving side strips it.
- The mnemonic "Please Do Not Throw Sausage Pizza Away" (Physical, Data Link, Network, Transport, Session, Presentation, Application) is widely recognized — useful for quick recall under pressure.
- In cloud discussions: "below L7" typically means NLB/L4 behavior; "L7" means ALB/Nginx/Envoy behavior.
