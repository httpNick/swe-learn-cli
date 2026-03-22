---
title: "Cloud Security"
tags: [security, iam, encryption, kms, secrets, waf, oauth2, jwt, zero-trust]
difficulty: medium
estimated_time: 25min
---

## Overview

Cloud security spans identity, data protection, network controls, and application-layer defenses.
In system design interviews, security is often an afterthought — mentioning it proactively and
concretely signals seniority. The core principle is defense in depth: multiple independent layers
so that no single failure exposes the system.

Identity and Access Management (IAM) controls who (or what) can do what to which resources.
The principle of least privilege means each user, service, or process gets only the permissions
it needs and nothing more. In AWS, prefer IAM roles over long-lived access keys — roles use
temporary credentials that rotate automatically. EC2 instance profiles and ECS task roles attach
roles directly to compute resources, eliminating the need to manage credentials in code.

Encryption protects data from unauthorized access. Encryption at rest uses AES-256 applied to
storage (S3 buckets, EBS volumes, RDS instances). Key management services (AWS KMS, GCP Cloud KMS)
handle key storage, rotation, and access control. Envelope encryption wraps a data encryption key
(DEK) with a master key (KEK) — only the DEK is stored alongside the data; the KEK lives in KMS.
Encryption in transit uses TLS 1.2 or 1.3 for all HTTP traffic. mTLS (mutual TLS) adds client
certificate verification for service-to-service communication in zero-trust environments.

OAuth2 and JWT handle delegated authorization and stateless authentication. OAuth2 defines the
flow for obtaining tokens (authorization code, client credentials). JWT encodes claims as a signed
(and optionally encrypted) JSON payload — the signature lets any service verify the token without
calling a central auth server. Always validate the signature, expiry (exp), issuer (iss), and
audience (aud) claims.

## When to Use

- **IAM roles/instance profiles**: any compute resource accessing cloud services — never hardcode
  access keys.
- **KMS envelope encryption**: sensitive data in S3, RDS, DynamoDB — automatic key rotation
  every year, auditable via CloudTrail.
- **Secrets Manager**: database passwords, API keys, OAuth client secrets — with automatic rotation
  and fine-grained access control. Use over Parameter Store when rotation is needed.
- **WAF**: public-facing APIs and web apps — SQL injection, XSS protection, rate limiting at
  the edge (before traffic reaches your servers).
- **JWT**: stateless service-to-service auth, mobile/SPA token-based auth where you don't want
  server-side session storage.
- **mTLS**: zero-trust service mesh environments (Istio, AWS App Mesh) where every service
  authenticates every other service.

## Trade-offs & Gotchas

- JWT tokens cannot be invalidated before expiry — keep access token TTL short (15min). Use
  refresh tokens (longer TTL, stored server-side) to issue new access tokens. A token blacklist
  defeats the stateless benefit; prefer short TTLs instead.
- KMS has API rate limits — at high request rates, use data key caching to avoid per-operation
  KMS calls.
- Overly broad IAM policies (Action: "*", Resource: "*") are the #1 cloud security mistake.
  Audit with IAM Access Analyzer.
- Secrets in environment variables are visible to anyone with exec access to the container. Use
  Secrets Manager injection at runtime via the AWS SDK or sidecar.
- WAF rules can introduce false positives — test in count mode before switching to block mode.
- TLS certificate expiry causes outages — use ACM (auto-renewing) or cert-manager (k8s) rather
  than manually managed certs.

## Architecture Diagram

```
  IAM Role Chain:
  [EC2 / Lambda]
       |
  [Instance Profile / Task Role]  (temporary creds, auto-rotated)
       |
  [IAM Policy] --> allow: s3:GetObject, kms:Decrypt
                   deny:  s3:DeleteObject (explicit deny wins)

  Envelope Encryption:
  [Plaintext Data]
       |
  [DEK (AES-256)] --> encrypt --> [Ciphertext]
       |
  [KMS: encrypt DEK with KEK] --> [Encrypted DEK stored with data]

  OAuth2 + JWT Flow:
  [Client] --> [Auth Server] --> [Access Token (JWT, 15min TTL)]
                                 [Refresh Token (opaque, 30 days)]
  [Client] --> [API] with Bearer JWT
  [API] --> validate JWT signature + exp + aud locally (no network call)

  Defense in Depth:
  Internet
    --> [WAF] (block SQLi, XSS, bad IPs)
    --> [ALB] (TLS termination)
    --> [App] (JWT validation, AuthZ)
    --> [KMS] (decrypt data at rest)
    --> [VPC] (private subnet, SG)
```

## Key Interview Points

- Least privilege: roles with minimum required permissions, no wildcard policies in production.
- No hardcoded credentials — use IAM roles, instance profiles, and Secrets Manager.
- Short-lived JWT access tokens (15min) + refresh tokens prevent token abuse without centralized
  revocation.
- Encrypt at rest (KMS) AND in transit (TLS 1.2+) — mention both when discussing data storage.
- WAF at the edge protects against OWASP Top 10 before traffic reaches your application.
- Zero trust: verify identity at every hop with mTLS — don't trust the private network implicitly.
