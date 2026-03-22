---
title: "DNS Resolution Flow"
tags: [dns, networking, resolution, records, ttl, routing]
difficulty: easy
estimated_time: 15min
---

## Overview

**DNS (Domain Name System)** translates human-readable domain names (`api.example.com`) into IP addresses. It is a globally distributed, hierarchically organized database. DNS is almost always the first step in any network request — a slow or misconfigured DNS can make an otherwise performant system feel broken. DNS records are cached at multiple levels (OS resolver, browser, recursive resolver) according to their **TTL (Time to Live)** value.

The resolution flow involves four actors. The **DNS recursive resolver** (provided by your ISP, Google `8.8.8.8`, Cloudflare `1.1.1.1`, or your corporate DNS) does the heavy lifting on the client's behalf. The **root nameservers** (13 logical clusters, operated by IANA) answer with the address of the **TLD nameserver** (e.g., `.com`, `.io`). The TLD nameserver answers with the **authoritative nameserver** for the domain, which holds the actual records (A, CNAME, MX, etc.) and returns the final answer.

Key record types: **A** (domain -> IPv4), **AAAA** (domain -> IPv6), **CNAME** (alias to another domain — cannot be used at zone apex), **ALIAS/ANAME** (like CNAME but allowed at apex — AWS Route 53 `ALIAS` record), **MX** (mail server), **TXT** (arbitrary text — SPF, DKIM, domain verification), **NS** (nameserver delegation), **SRV** (service discovery, used by some protocols), **PTR** (reverse DNS — IP to hostname).

```
DNS Resolution: Querying "api.example.com" (cache miss):

  [Browser / App]
       |
  1. Check OS cache (hosts file, /etc/hosts, nscd)
       |  (cache miss)
  2. Query Local Recursive Resolver (e.g., 1.1.1.1)
       |  (cache miss at resolver)
  3. Resolver queries Root Nameserver (".")
       |  <-- "ask .com TLD at 192.5.6.30"
  4. Resolver queries .com TLD Nameserver
       |  <-- "ask example.com NS at 205.251.196.1"
  5. Resolver queries example.com Authoritative NS
       |  <-- "api.example.com A 93.184.216.34 TTL=300"
  6. Resolver caches and returns answer to client
       |
  [Browser connects to 93.184.216.34]

  Total round trips: ~3-4 (all cached after first resolution)
  Cached resolution: ~1ms (resolver cache hit)
  Full recursive resolution: ~50-150ms

Record Types Quick Reference:
  A       example.com   -> 93.184.216.34        (IPv4)
  AAAA    example.com   -> 2606:2800:220:1::...  (IPv6)
  CNAME   api.example.com -> lb.example.com     (alias, no apex)
  ALIAS   example.com   -> lb.example.com       (apex-safe CNAME)
  MX      example.com   -> mail.example.com     (email)
  TXT     example.com   -> "v=spf1 include:..."  (SPF/DKIM)
  NS      example.com   -> ns1.example.com      (delegation)
```

## When to Use

- **Low TTL (30-60s)**: During deployments, blue/green switches, or failover events where you need DNS changes to propagate quickly. Low TTL increases resolver query volume.
- **High TTL (300-3600s)**: Stable infrastructure where change is infrequent. Reduces resolver load and improves client-side latency (more cache hits).
- **Route 53 / Cloud DNS for latency routing**: Route users to the nearest region based on their resolver's location. Combine with health checks to automate failover.
- **Private DNS / internal DNS**: Route internal service names (`payments-service.internal`) to private IPs without exposing them publicly. AWS Route 53 Private Hosted Zones, CoreDNS in Kubernetes.
- **CNAME vs ALIAS**: Use ALIAS at the zone apex (naked domain `example.com`) pointing to an ALB or CloudFront. CNAME cannot be used at the apex due to RFC constraints.

## Trade-offs & Gotchas

- **DNS propagation is not instant**: TTL controls how long resolvers cache records. To reduce downtime during a cutover, lower TTL to 60s 24-48 hours *before* the change; raise it back after the change stabilizes.
- **Negative TTL (NXDOMAIN caching)**: If a hostname doesn't exist, resolvers cache that negative response for the SOA record's `MINIMUM` TTL. Creating a new record doesn't immediately fix "NXDOMAIN" errors seen by clients.
- DNS does not guarantee a single IP — responses can include multiple A records. Clients typically use the first; the OS or resolver may rotate them (round-robin DNS). This is a rudimentary load balancing mechanism.
- **Split-horizon DNS**: Different answers for internal vs external clients. Used to expose `api.example.com` as a public IP externally and a private IP internally.
- DNS over HTTPS (DoH) and DNS over TLS (DoT) encrypt DNS queries — prevents ISP snooping and manipulation, but may conflict with corporate DNS filtering.
- `dig +trace api.example.com` walks the full resolution chain from root — the essential debugging tool. `nslookup` is available everywhere but less informative.
- DNS is UDP by default (port 53). Responses > 512 bytes fall back to TCP. DNSSEC and large TXT records frequently trigger TCP fallback.

## Key Points for Interviews

- DNS resolution: client -> recursive resolver -> root NS -> TLD NS -> authoritative NS -> answer (3-4 round trips on cache miss, ~1ms on cache hit).
- TTL controls propagation speed. Lower TTL before planned changes; raise it back after.
- Record types: A (IPv4), AAAA (IPv6), CNAME (alias, not at apex), ALIAS (apex-safe), MX (mail), TXT (SPF/DKIM/verification), NS (delegation).
- Route 53 health checks + DNS failover = auto-routing around failed regions.
- Private hosted zones / CoreDNS handle service discovery in VPCs and Kubernetes clusters.
- Debugging commands: `dig +trace`, `nslookup`, `host`. Always check TTL when records seem stale.
- DNS is a common single point of failure — mention redundant resolvers, Anycast-based DNS providers (Route 53, Cloudflare), and low TTL strategy for failover.
