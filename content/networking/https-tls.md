---
title: "HTTPS and TLS Handshake"
tags: [https, tls, ssl, security, encryption, certificates, pki]
difficulty: medium
estimated_time: 20min
---

## Overview

**HTTPS** is HTTP over **TLS (Transport Layer Security)**. TLS provides three guarantees: **confidentiality** (traffic is encrypted, unreadable to eavesdroppers), **integrity** (traffic cannot be tampered with undetected via MAC/AEAD), and **authentication** (the server's identity is verified via a certificate signed by a trusted CA). TLS 1.3 (RFC 8446, 2018) is the current standard; TLS 1.0 and 1.1 are deprecated and disabled in modern clients.

The **TLS handshake** establishes a shared session key using asymmetric cryptography, then switches to symmetric encryption for the actual data transfer. In TLS 1.3, this is accomplished in a single round trip (1-RTT): the client sends supported cipher suites and a key share in the `ClientHello`; the server responds with its chosen cipher, its own key share, and its certificate; the client verifies the certificate against trusted CAs and both sides derive the same session key via **ECDHE (Elliptic Curve Diffie-Hellman Ephemeral)**. No private key material is ever transmitted — **forward secrecy** ensures that compromising the server's private key later cannot decrypt past sessions.

A **certificate** binds a public key to a domain name, signed by a **Certificate Authority (CA)**. The browser/OS ships with a list of trusted root CAs. Intermediate CAs form a chain of trust. **Let's Encrypt** provides free DV (Domain Validation) certificates via the ACME protocol. **mTLS (mutual TLS)** extends this: the client also presents a certificate, enabling strong service-to-service authentication in zero-trust architectures (used by service meshes like Istio/Envoy).

```
TLS 1.3 Handshake (1-RTT):

  Client                              Server
    |                                    |
    |  ------- ClientHello ----------->  |
    |  (supported ciphers, key_share,    |
    |   TLS version, random nonce)       |
    |                                    |
    |  <------ ServerHello -----------  |
    |  (chosen cipher, key_share,        |
    |   Certificate, CertificateVerify, |
    |   Finished)                        |
    |                                    |
    |  (client verifies cert chain,      |
    |   both derive session key via ECDHE)|
    |                                    |
    |  ------- Finished + HTTP req --->  |  <-- data starts here (1-RTT)
    |  <------ HTTP response ----------  |

TLS 1.3 0-RTT (Session Resumption):
  Client sends PSK (pre-shared key from previous session)
  + early data (HTTP request) in ClientHello
  Server can respond immediately if PSK is valid
  Risk: 0-RTT data has no replay protection -- avoid for non-idempotent requests

Certificate Chain of Trust:
  Root CA (trusted by OS/browser)
      |
  Intermediate CA (signed by root)
      |
  Server Certificate (signed by intermediate,
                      contains domain name + public key)
```

## When to Use

- **HTTPS everywhere**: There is no legitimate reason to serve public traffic over plain HTTP in 2024. Browsers mark HTTP sites as insecure, and HTTP/2 requires TLS.
- **mTLS**: Service-to-service authentication in microservices / zero-trust networks. Service mesh sidecars (Envoy/Istio) handle mTLS transparently without application code changes.
- **TLS termination at the load balancer**: Decrypt at the edge (ALB, Nginx), forward plaintext over a trusted internal network. Simpler certificate management, backend servers handle less crypto load.
- **End-to-end TLS (re-encryption)**: Terminate and re-encrypt between each hop. Required for compliance scenarios (PCI-DSS, HIPAA) where traffic must be encrypted even on internal networks.
- **Certificate pinning**: Mobile apps that pin the expected certificate or public key to prevent MITM via rogue CAs. Use with care — a certificate rotation without updating the app breaks it.

## Trade-offs & Gotchas

- TLS 1.3 is strictly better than TLS 1.2: faster (1-RTT vs 2-RTT), removed weak cipher suites (RC4, 3DES, RSA key exchange without forward secrecy). Always enforce TLS 1.2 minimum; prefer TLS 1.3 only.
- **OCSP stapling**: Instead of the client querying the CA's OCSP server to check if a certificate is revoked (adding latency and a privacy leak), the server periodically fetches and caches ("staples") the OCSP response and sends it during the TLS handshake.
- **SNI (Server Name Indication)**: TLS extension that allows the client to specify the hostname in the ClientHello before the handshake completes, enabling a single IP to host multiple TLS certificates (virtual hosting). Without SNI, you need one IP per certificate.
- **Wildcard certificates** (`*.example.com`) cover one subdomain level. They cannot cover `api.v2.example.com`. **SAN (Subject Alternative Name)** certificates explicitly list multiple domains.
- Certificate rotation is an operational concern: automate with Let's Encrypt + Certbot or AWS Certificate Manager (ACM). ACM auto-rotates certs on ALB/CloudFront with zero downtime.
- `HSTS (HTTP Strict Transport Security)` header tells browsers to only connect via HTTPS for a given duration — prevents SSL-stripping attacks. Include `includeSubDomains` and preload to `hstspreload.org` for maximum effect.
- CPU cost of TLS is negligible on modern hardware with AES-NI instructions — do not omit TLS for "performance" reasons.

## Key Points for Interviews

- TLS provides confidentiality, integrity, and server authentication. mTLS adds client authentication.
- TLS 1.3 handshake = 1-RTT (vs TLS 1.2's 2-RTT). 0-RTT is possible on session resumption but vulnerable to replay attacks.
- ECDHE provides **forward secrecy** — compromising the private key does not decrypt past sessions.
- TLS termination at the load balancer is standard; re-encryption (end-to-end TLS) is required for stricter compliance scenarios.
- Certificate chain: server cert -> intermediate CA -> root CA. Browser trusts the root, validates the chain.
- SNI enables multiple domains on one IP. OCSP stapling reduces handshake latency. HSTS prevents downgrade attacks.
- In zero-trust / service mesh architectures, mTLS is the standard — mention Istio, Envoy, or Linkerd.
