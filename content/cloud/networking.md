---
title: "Cloud Networking"
tags: [vpc, subnets, nat, cdn, dns, networking, security-groups]
difficulty: medium
estimated_time: 25min
---

## Overview

A Virtual Private Cloud (VPC) is a logically isolated network within a cloud provider where you
launch resources. You define the IP address space (CIDR block, e.g. 10.0.0.0/16), divide it into
subnets, and control traffic with routing tables and firewalls. Every production system lives inside
a VPC — understanding its structure is foundational to any cloud architecture answer.

Subnets are subdivisions of the VPC CIDR, each pinned to a single Availability Zone. Public subnets
have a route to an Internet Gateway (IGW), so resources in them can receive inbound internet
traffic. Private subnets have no direct internet route — outbound-only access goes through a NAT
Gateway. Best practice: put application servers and databases in private subnets, and only expose
load balancers in public subnets.

DNS and CDNs sit at the edge of your architecture. Route 53 (or equivalent) resolves domain names
and can implement intelligent routing policies to direct users to the nearest or healthiest region.
CDNs push static and cacheable content to edge locations worldwide, reducing latency and origin
load.

## When to Use

In every system design answer involving cloud infrastructure, establish the network topology early:
- "I'd put the API servers in private subnets behind an ALB in the public subnet."
- Mention NAT Gateway when private instances need to pull packages or call external APIs.
- Use Route 53 latency-based routing when designing multi-region systems.
- Propose a CDN (CloudFront, Fastly) whenever serving static assets, video, or geographically
  distributed users.

## Trade-offs & Gotchas

- NAT Gateway is a managed single-AZ service — deploy one per AZ for HA, or accept the AZ
  dependency. Cost adds up at high egress volumes.
- VPC Peering is non-transitive: if A peers with B and B peers with C, A cannot reach C.
  Use Transit Gateway for hub-and-spoke multi-VPC topologies.
- Security Groups are stateful (return traffic is automatically allowed). NACLs are stateless
  (you must explicitly allow both inbound and outbound). NACLs operate at the subnet level.
- CDN cache invalidation is slow and costly — design URLs to be cache-busted by version
  (e.g., /assets/app.v3.js) rather than relying on invalidation.
- Route 53 TTLs affect failover speed: low TTL = faster failover, higher DNS query cost.
- IPv4 CIDR planning is permanent — plan for growth. A /16 gives 65k addresses; /24 gives 256.

## Architecture Diagram

```
                         Internet
                            |
                    [Internet Gateway]
                            |
            +---------------+---------------+
            |                               |
    [Public Subnet AZ-1]           [Public Subnet AZ-2]
    [ALB / Bastion]                [ALB / Bastion]
            |                               |
    [NAT Gateway]                  [NAT Gateway]
            |                               |
    [Private Subnet AZ-1]          [Private Subnet AZ-2]
    [App Servers]                  [App Servers]
            |                               |
    [Private Subnet AZ-1]          [Private Subnet AZ-2]
    [RDS Primary]                  [RDS Replica]

    VPC Peering / Transit Gateway connects to other VPCs
    Route 53 routes DNS -> ALB (latency/failover/weighted)
    CDN (CloudFront) sits in front of ALB or S3 origin
```

## Key Interview Points

- Always put databases and app servers in private subnets — only LBs go in public subnets.
- One NAT Gateway per AZ for high availability; cross-AZ NAT traffic incurs data transfer costs.
- VPC Peering for simple two-VPC connectivity; Transit Gateway for 3+ VPCs or transitive routing.
- Route 53 routing policies: latency-based (performance), failover (HA), geolocation (compliance),
  weighted (canary deploys / A/B testing).
- CDN reduces origin load and latency — mention it whenever serving media, SPAs, or global users.
- Security Groups = instance-level stateful firewall. NACLs = subnet-level stateless firewall.
  Use SGs as your primary control; NACLs for broad subnet-level denies.
