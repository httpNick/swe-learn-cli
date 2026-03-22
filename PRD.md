# Product Requirements Document: SWE Learn CLI

**Version:** 0.1 (Draft)
**Date:** 2026-03-20
**Status:** In Progress

---

## 1. Overview

**SWE Learn** is an offline, terminal-based learning tool built in Rust that helps software engineers prepare for technical interviews. It provides an interactive TUI (Terminal User Interface) for navigating structured study content across core software engineering domains — with a focus on system design, cloud architecture, and computer science fundamentals.

**Vision:** A single command gets you into a focused, distraction-free study environment. No browser, no internet, no ads.

---

## 2. Problem Statement

Engineers preparing for software engineering interviews face scattered learning resources: blog posts, YouTube videos, paid courses, and flashcard apps that are fragmented across platforms and often require an internet connection. There is no unified, keyboard-driven, offline CLI tool that surfaces the right content at the right depth for technical interview prep.

**Key pain points:**
- Resources are scattered and inconsistent in quality
- Most tools require internet access
- No fast, keyboard-driven reference that fits into a terminal workflow
- Mermaid/architecture diagrams aren't available in a native CLI format
- No single tool covers system design + cloud architecture + CS fundamentals together

---

## 3. Target Users

**Primary:** Mid-level to senior software engineers actively preparing for system design and technical interviews at top tech companies (FAANG, MANGA, and similar).

**Secondary:** Junior engineers building foundational knowledge and students transitioning into professional roles.

**User characteristics:**
- Comfortable in the terminal
- Already know how to code; need to deepen architecture and design knowledge
- Preparing for interviews within a defined timeframe (weeks to months)
- Want quick-reference content, not textbook-length prose

---

## 4. Goals

### In Scope (MVP)
- TUI with keyboard-driven navigation across learning modules
- Cloud Architecture module covering key patterns and services
- System Design Questions module with Mermaid diagrams for each design
- Databases module covering SQL/NoSQL, indexing, replication, and consistency models
- Networking & Protocols module covering HTTP, TCP/IP, DNS, CDNs, and load balancing
- Data Structures & Algorithms module with complexity references and key patterns
- DevOps & CI/CD module covering containers, Kubernetes, observability, and SRE practices
- Fully offline — all content bundled with the binary
- Fast startup (< 200ms to interactive)
- Cross-platform: Linux, macOS, Windows (WSL)

### Out of Scope (MVP)
- User accounts or progress sync
- AI-generated explanations or LLM integration
- Spaced repetition / flashcard mode
- External content updates or plugin system
- Mobile or web interface

---

## 5. Features

### 5.1 TUI Shell

The top-level interface users land on after running `swelearn`.

**Behavior:**
- Full-screen terminal UI built with `ratatui`
- Module selection menu with descriptions
- Consistent keybindings across all screens
- Status bar showing current location and available keybindings

**Keybindings:**
| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate list |
| `Enter` | Select / drill down |
| `Esc` / `b` | Go back |
| `/` | Search within module |
| `q` | Quit |
| `?` | Show help overlay |

**Layout:**
```
┌─ SWE Learn ─────────────────────────────────────────────┐
│                                                          │
│  > Cloud Architecture                                    │
│    System Design Questions                               │
│    Databases                                             │
│    Networking & Protocols                                │
│    Data Structures & Algorithms                          │
│    DevOps & CI/CD                                        │
│                                                          │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  [↑↓/jk] Navigate   [Enter] Open   [/] Search   [q] Quit│
└──────────────────────────────────────────────────────────┘
```

---

### 5.2 Cloud Architecture Module

Covers the building blocks of distributed systems and cloud infrastructure commonly referenced in system design interviews.

**Content areas:**

| Category | Topics |
|----------|--------|
| Compute | Virtual machines, containers, serverless functions, auto-scaling |
| Storage | Object storage, block storage, file systems, data lakes |
| Networking | VPCs, subnets, NAT gateways, peering, DNS, CDNs |
| Databases | Managed relational, managed NoSQL, data warehouses, caching layers |
| Messaging | Message queues (SQS/Kafka/RabbitMQ), pub/sub (SNS/Pub/Sub), event streaming, dead letter queues |
| Load Balancing | L4 vs L7, ALB, NLB, global load balancing, health checks, algorithms |
| Caching | Redis, Memcached, cache-aside, write-through, write-behind, eviction policies, multi-level |
| Security | IAM, encryption at rest/in transit, secrets management, WAF, OAuth2/JWT |
| Observability | Metrics, logs, traces, alerting, dashboards (Prometheus, Grafana, ELK, Jaeger) |
| Multi-region | Active-active vs active-passive, failover, replication lag |
| Resilience Patterns | Circuit breaker, bulkhead, retry with exponential backoff, timeouts, backpressure |
| Search & Indexing | Inverted indexes, full-text search, geospatial indexing, autocomplete (Elasticsearch, Algolia) |

**Each topic includes:**
- Concept explanation (2–4 paragraphs)
- When to use it (interview context)
- Trade-offs and gotchas
- ASCII or Mermaid diagram where applicable

---

### 5.3 System Design Questions Module

A curated list of common system design interview questions with structured answers and architecture diagrams.

**Format for each question:**
1. **Problem statement** — the interview prompt as typically asked
2. **Clarifying questions** — what to ask the interviewer first
3. **High-level design** — architecture overview with Mermaid diagram
4. **Deep dives** — scalability, storage estimation, bottlenecks, trade-offs
5. **Key decisions** — what choices to highlight and why

**Question set — tiered by interview frequency:**

**Tier 1 — Appears in 80%+ of interviews (MVP scope)**

| # | Question | Key Concepts | Common At |
|---|----------|-------------|-----------|
| 1 | Design a URL Shortener (e.g., bit.ly) | Hashing, collision handling, redirects, DB design | Google, Amazon, Meta |
| 2 | Design Twitter / a social media feed | Timeline fanout, caching, eventual consistency | Meta, Amazon, Google |
| 3 | Design a web crawler | Distributed systems, politeness policies, DNS | Google, Bing, Amazon |
| 4 | Design YouTube / video streaming | Video encoding, CDN, streaming protocols, caching | Google, Meta, Netflix |
| 5 | Design a ride-sharing service (e.g., Uber) | Geospatial indexing, real-time matching, payments | Uber, Amazon, Apple |
| 6 | Design a chat application (e.g., WhatsApp) | Message queues, real-time comms, persistence | Meta, Google, Discord |
| 7 | Design a file storage service (e.g., Dropbox) | File sync, versioning, conflict resolution, dedup | Google, Microsoft, Dropbox |
| 8 | Design a notification system | Message queues, push notifications, user prefs | Google, Meta, Amazon |
| 9 | Design a rate limiter | Token bucket, sliding window, distributed limits | Google, Stripe, Amazon |
| 10 | Design a distributed cache (e.g., Redis) | Eviction policies, TTL, consistent hashing | Google, Amazon, Meta |

**Tier 2 — Appears in 50–80% of interviews (Phase 2 scope)**

| # | Question | Key Concepts | Common At |
|---|----------|-------------|-----------|
| 11 | Design a search autocomplete system | Prefix trees, ranking, caching | Google, Microsoft, Amazon |
| 12 | Design a news feed algorithm | Ranking, personalization, real-time updates | Meta, TikTok, Google |
| 13 | Design a payment gateway | Transactions, fraud detection, PCI compliance | Stripe, PayPal, Amazon |
| 14 | Design a hotel reservation system | Availability management, pricing, search | Airbnb, Booking.com |
| 15 | Design a proximity service (e.g., Yelp) | Geospatial indexing, search, reviews | Google, Yelp, Amazon |
| 16 | Design a leaderboard / ranking system | Sorted sets, real-time updates, scalability | Microsoft, Amazon, gaming |
| 17 | Design a distributed message queue | Partitioning, ordering, consumer groups | Google, Amazon, Meta |
| 18 | Design a distributed ID generator (e.g., Snowflake) | Clock synchronization, uniqueness guarantees | Twitter, Discord, Amazon |
| 19 | Design a data analytics platform | Distributed storage, MapReduce, query optimization | Google, Amazon, Meta |
| 20 | Design an e-commerce checkout system | Transactions, inventory, payment processing | Amazon, eBay, Shopify |

**Mermaid diagram rendering:** Diagrams are stored as Mermaid syntax in content files. The TUI renders them as ASCII art inline using a Mermaid-to-ASCII conversion (via the `mermaid-ascii` crate or similar). Alternatively, diagrams can be rendered and cached at build time.

---

### 5.4 Databases Module

**Topics:**
- SQL vs NoSQL: when to choose each, trade-offs
- Indexing: B-tree, hash indexes, composite indexes, covering indexes
- Replication: leader-follower, multi-leader, leaderless, quorum-based
- Sharding: horizontal vs vertical, consistent hashing, shard key selection
- Transactions & ACID properties
- CAP theorem and its practical implications
- PACELC theorem: extension of CAP adding latency trade-offs
- Consistency models: eventual, strong, causal, read-your-writes
- CQRS: Command Query Responsibility Segregation pattern
- Event Sourcing: storing state changes as an ordered event log
- Common databases compared: PostgreSQL, MySQL, MongoDB, Cassandra, DynamoDB, Redis, HBase

---

### 5.5 Networking & Protocols Module

**Topics:**
- OSI model: quick reference layer-by-layer
- TCP vs UDP: when each is appropriate
- HTTP/1.1 vs HTTP/2 vs HTTP/3
- HTTPS and TLS handshake
- DNS resolution flow
- Load balancing algorithms: round robin, least connections, consistent hashing, geographic
- CDN: edge caching, cache invalidation, origin pull vs push, HLS/DASH for video
- WebSockets and long polling
- gRPC vs REST vs GraphQL: protocol trade-offs
- Reverse proxies and API gateways (Kong, Apigee, AWS API Gateway)

---

### 5.6 Data Structures & Algorithms Module

Focused on interview-relevant patterns, not academic completeness.

**Content:**
- Big-O complexity cheat sheet (time and space)
- Core data structures: arrays, linked lists, stacks, queues, heaps, tries, graphs, trees
- Sorting algorithms: comparison and complexity
- Key algorithm patterns: sliding window, two pointers, BFS/DFS, dynamic programming, backtracking, binary search
- Common interview problem categories with pattern recognition tips

---

### 5.7 DevOps & CI/CD Module

**Topics:**
- Containers: Docker architecture, layering, image best practices
- Kubernetes: pods, services, deployments, ingress, HPA, resource limits
- CI/CD pipelines: stages, artifacts, environments, rollback strategies
- Infrastructure as Code: Terraform, Pulumi concepts
- Observability: the three pillars (metrics, logs, traces), SLI/SLO/SLA
- SRE practices: error budgets, toil reduction, incident response
- Blue/green and canary deployments

---

## 6. Content Architecture

### Storage Format

All content is stored as structured Markdown files bundled into the binary at compile time using Rust's `include_str!` macro or embedded via a build script.

```
content/
  cloud/
    compute.md
    storage.md
    networking.md
    ...
  system-design/
    url-shortener.md
    twitter-feed.md
    ...
  databases/
    ...
  networking/
    ...
  algorithms/
    ...
  devops/
    ...
```

### Content Schema (per topic file)

```markdown
---
title: "Design a URL Shortener"
tags: [hashing, databases, caching, api-design]
difficulty: medium
estimated_time: 45min
---

## Problem Statement
...

## Clarifying Questions
...

## High-Level Design

```mermaid
graph TD
    Client --> API_Gateway
    API_Gateway --> Write_Service
    API_Gateway --> Read_Service
    Write_Service --> DB[(SQL DB)]
    Read_Service --> Cache[(Redis)]
    Cache --> DB
\```

## Deep Dives
...
```

---

## 7. UX Inspiration

These existing CLI and TUI tools inform the design of SWE Learn:

| Tool | Language | What to Borrow |
|------|----------|----------------|
| **lazygit** | Go | Keyboard-driven TUI with discoverable bindings shown in a footer bar; visual hierarchy through panes |
| **tealdeer** | Rust | Offline-first CLI reference; `cargo install` distribution; fast cold start |
| **navi** | Rust | Interactive guided selection — useful model for "study mode" where users are prompted interactively |
| **fzf** | Go | Real-time fuzzy filtering as the user types — the gold standard for `/` search UX |
| **bat** | Rust | Syntax-highlighted markdown rendering in the terminal — relevant for content display |
| **tldr** | Various | Concise, community-maintained content format; good template for how to write topic files |

**Key UX principles derived from research:**
- Minimal input → maximum value (tldr model)
- All keybindings discoverable from the footer (lazygit model)
- Real-time filter feedback on `/` search (fzf model)
- No required internet connection (tealdeer model)

---

## 8. Technical Architecture


### Tech Stack

| Concern | Choice | Rationale |
|---------|--------|-----------|
| Language | Rust | Performance, binary portability, no runtime dependency |
| TUI framework | `ratatui` | Most mature Rust TUI library, active community |
| Markdown rendering | `termimad` or `pulldown-cmark` | Render markdown to styled terminal output |
| Mermaid diagrams | Pre-rendered ASCII at build time | No runtime JS dependency |
| Content storage | Embedded files via `include_str!` | Single binary, fully offline |
| Search | In-memory fuzzy search (`nucleo` or `skim`) | Fast, no index files |
| Config | `~/.config/swelearn/config.toml` | XDG-compliant |

### Binary Distribution

- GitHub Releases with pre-built binaries for Linux (x86_64, arm64), macOS (x86_64, arm64), Windows
- `cargo install swelearn` via crates.io
- Homebrew formula (post-MVP)

### Performance Targets

| Metric | Target |
|--------|--------|
| Binary size | < 15 MB |
| Startup to interactive | < 200ms |
| Search response | < 50ms |
| Memory usage | < 50 MB |

---

## 9. User Experience

### First Run

```
$ swelearn

Welcome to SWE Learn! Navigate with ↑↓, select with Enter, quit with q.
Press ? for help at any time.

[Module selection menu appears]
```

### Search

Pressing `/` from any screen opens a fuzzy search across all content titles and tags. Results filter in real-time.

### Progress Tracking (Post-MVP)

A local `~/.config/swelearn/progress.json` file tracks which topics and questions have been viewed. A progress indicator (`[✓]`) appears next to visited items.

---

## 10. MVP Scope & Phasing

### Phase 1 — MVP (Target: 8 weeks)
- [x] TUI shell with module navigation
- [x] System Design Questions module (10 questions) *(10/10 complete)*
- [ ] Cloud Architecture module (core topics)
- [ ] Databases module
- [ ] Networking module
- [ ] Binary builds for Linux and macOS
- [ ] `cargo install` support

### Phase 2 — Content Complete
- [ ] All 20 system design questions
- [ ] Data Structures & Algorithms module
- [ ] DevOps & CI/CD module
- [ ] Windows/WSL support
- [ ] Full-text search across all content
- [ ] Homebrew formula

### Phase 3 — Enhanced UX
- [ ] Progress tracking (visited items)
- [ ] Bookmarks / favorites
- [ ] Study mode: random question selection
- [ ] Content versioning and update mechanism
- [ ] Configurable themes (light/dark terminal themes)

---

## 11. Success Metrics

Since this is a developer tool (likely open source), success is measured by:

| Metric | MVP Target | 6-Month Target |
|--------|-----------|----------------|
| GitHub stars | 100 | 1,000 |
| `cargo install` installs | 50 | 500 |
| Content coverage | 10 SD questions | 20+ SD questions |
| Startup time | < 200ms | < 100ms |
| Open issues resolved | — | > 80% within 2 weeks |

---

## 12. Open Questions

- [x] **Mermaid rendering:** Pre-render diagrams to ASCII at build time via a Node.js build script. Node.js is a build-only dependency — not required at runtime. Output is cached as static strings embedded in the binary.
- [x] **License:** MIT.
- [ ] Should content be editable by users (custom notes per topic)? *(Deferred to Phase 3)*
- [ ] Is there appetite for a community content contribution model (GitHub PRs for new questions)? *(Deferred — post-launch)*
- [ ] Should DSA section include runnable code examples (embedded interpreter or link to playground)? *(Deferred to Phase 2 — link to playground for now)*

---

*This document is a living draft. Open questions should be resolved before implementation begins on each phase.*
