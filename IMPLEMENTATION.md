# Implementation Details

This document covers concrete technology choices and dependencies. For the architectural design, see [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Cryptography

### Authentication
- **Signature scheme:** ed25519

### Encryption at Rest
- **Scheme:** Symmetric AEAD — currently **XChaCha20-Poly1305**
- **Version tracking:** Each `aead_encrypted` database entry carries a `scheme` field denoting the version, enabling transparent migration on unseal

### Server Identity
- **Transport:** TLS with a self-signed certificate
- **Key type:** Generated on first run; long-term (no rotation mechanism yet)

---

## Communication

- **Protocol:** gRPC with Protocol Buffers
- **Server identity distribution:** `ServerInfo` protobuf struct containing the TLS public key fingerprint
- **Future consideration:** grpc-web lacks bidirectional stream support, so a browser-based wallet may require protojson over WebSocket

---

## Memory Protection

The unsealed root key must be held in a hardened memory cell resistant to dumps, page swaps, and hibernation.

- **Current:** Using the `memsafe` crate as an interim solution
- **Planned:** Custom implementation based on `mlock` (Unix) and `VirtualProtect` (Windows)
