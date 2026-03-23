# swelearn

An offline, keyboard-driven terminal tool for software engineering interview prep.

No browser. No internet. No ads. Just content.

```
┌─ SWE Learn ─────────────────────────────────────────────┐
│                                                          │
│  > Cloud Architecture                                    │
│    System Design Questions                               │
│    Databases                                             │
│    Networking & Protocols                                │
│                                                          │
├──────────────────────────────────────────────────────────┤
│  [↑↓/jk] Navigate   [Enter] Open   [/] Search   [q] Quit│
└──────────────────────────────────────────────────────────┘
```

## Install

**Via Cargo:**
```
cargo install swelearn
```

**Pre-built binary** — download from [GitHub Releases](https://github.com/httpNick/swe-learn-cli/releases) for Linux (x86_64, arm64), macOS (x86_64, arm64), or Windows.

## Usage

```
swelearn
```

That's it. Navigate with the keyboard.

| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate |
| `Enter` | Select |
| `Esc` / `b` | Go back |
| `/` | Search |
| `q` | Quit |
| `?` | Help |

## Content

- **System Design Questions** — 10 common interview questions (URL shortener, Twitter feed, YouTube, Uber, WhatsApp, web crawler, Dropbox, notifications, rate limiter, distributed cache) with architecture diagrams, clarifying questions, deep dives, and trade-offs
- **Cloud Architecture** — Compute, storage, networking, databases, messaging, load balancing, caching, security, observability, resilience patterns, multi-region design
- **Databases** — SQL vs NoSQL, indexing, replication, sharding, ACID, CAP/PACELC, consistency models, CQRS, event sourcing
- **Networking & Protocols** — OSI model, TCP/UDP, HTTP versions, TLS, DNS, CDNs, WebSockets, gRPC vs REST vs GraphQL

All content is bundled into the binary — nothing is downloaded at runtime.

## License

MIT
