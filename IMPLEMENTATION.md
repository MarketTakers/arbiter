# Implementation Details

This document covers concrete technology choices and dependencies. For the architectural design, see [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Client Connection Flow

### New Client Approval

When a client whose public key is not yet in the database connects, all connected user agents are asked to approve the connection. The first agent to respond determines the outcome; remaining requests are cancelled via a watch channel.

```mermaid
flowchart TD
    A([Client connects]) --> B[Receive AuthChallengeRequest]
    B --> C{pubkey in DB?}

    C -- yes --> D[Read nonce\nIncrement nonce in DB]
    D --> G

    C -- no --> E[Ask all UserAgents:\nClientConnectionRequest]
    E --> F{First response}
    F -- denied --> Z([Reject connection])
    F -- approved --> F2[Cancel remaining\nUserAgent requests]
    F2 --> F3[INSERT client\nnonce = 1]
    F3 --> G[Send AuthChallenge\nwith nonce]

    G --> H[Receive AuthChallengeSolution]
    H --> I{Signature valid?}
    I -- no --> Z
    I -- yes --> J([Session started])
```

### Known Issue: Concurrent Registration Race (TOCTOU)

Two connections presenting the same previously-unknown public key can race through the approval flow simultaneously:

1. Both check the DB → neither is registered.
2. Both request approval from user agents → both receive approval.
3. Both `INSERT` the client record → the second insert silently overwrites the first, resetting the nonce.

This means the first connection's nonce is invalidated by the second, causing its challenge verification to fail. A fix requires either serialising new-client registration (e.g. an in-memory lock keyed on pubkey) or replacing the separate check + insert with an `INSERT OR IGNORE` / upsert guarded by a unique constraint on `public_key`.

### Nonce Semantics

The `program_client.nonce` column stores the **next usable nonce** — i.e. it is always one ahead of the nonce last issued in a challenge.

- **New client:** inserted with `nonce = 1`; the first challenge is issued with `nonce = 0`.
- **Existing client:** the current DB value is read and used as the challenge nonce, then immediately incremented within the same exclusive transaction, preventing replay.

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
