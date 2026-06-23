# Implementation Details

This document covers concrete technology choices and dependencies. For the architectural design, see [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Client Connection Flow

### Authentication Result Semantics

Authentication no longer uses an implicit success-only response shape. Both `client` and `operator` return explicit auth status enums over the wire.

- **Client:** `AuthResult` may return `SUCCESS`, `INVALID_KEY`, `INVALID_SIGNATURE`, `APPROVAL_DENIED`, `NO_OPERATORS_ONLINE`, or `INTERNAL`
- **Operator:** `AuthResult` may return `SUCCESS`, `INVALID_KEY`, `INVALID_SIGNATURE`, `BOOTSTRAP_REQUIRED`, `TOKEN_INVALID`, or `INTERNAL`

This makes transport-level failures and actor/domain-level auth failures distinct:

- **Transport/protocol failures** are surfaced as stream/status errors
- **Authentication failures** are surfaced as successful protocol responses carrying an explicit auth status

Clients are expected to handle these status codes directly and present the concrete failure reason to the user.

### New Client Approval

When a client whose public key is not yet in the database connects, all connected operators are asked to approve the connection. The first operator to respond determines the outcome; remaining requests are cancelled via a watch channel.

```mermaid
flowchart TD
    A([Client connects]) --> B[Receive AuthChallengeRequest]
    B --> C{pubkey in DB?}

    C -- yes --> G[Generate AuthChallenge]

    C -- no --> E[Ask all Operators:\nClientConnectionRequest]
    E --> F{First response}
    F -- denied --> Z([Reject connection])
    F -- approved --> F2[Cancel remaining\nOperator requests]
    F2 --> F3[INSERT client]
    F3 --> G

    G --> H[Send AuthChallenge\ntimestamp + random bytes]
    H --> I[Receive AuthChallengeSolution]
    I --> K{Signature valid?}
    K -- no --> Z
    K -- yes --> J([Session started])
```

Auth challenges are generated from fresh random bytes plus a nanosecond timestamp. The server keeps the issued challenge only in the in-flight authentication state for that connection, then verifies the signature against the same canonical challenge payload.

The authentication schema stores peer identity, not replay counters:

- `program_client` stores the SDK client's public key, metadata binding, and timestamps.
- `operator_client` stores the Operator public key and timestamps.
- Neither table stores an authentication nonce, and challenge generation does not update either table.

---

## Cryptography

### Authentication
- **Client protocol:** ML-DSA

### User-Agent Authentication

Operator authentication supports multiple signature schemes because platform-provided "hardware-bound" keys do not expose a uniform algorithm across operating systems and hardware.

- **Supported schemes:** ML-DSA
- **Why:** Secure Enclave (MacOS) support them natively, on other platforms we could emulate while they roll-out

### Encryption at Rest
- **Scheme:** Symmetric AEAD — currently **XChaCha20-Poly1305**
- **Version tracking:** Each `aead_encrypted` database entry carries a `scheme` field denoting the version, enabling transparent migration on unseal

### Server Identity
- **Transport:** TLS with a self-signed certificate
- **Key type:** Generated on first run; long-term (no rotation mechanism yet)

---

## Communication

- **Protocol:** gRPC with Protocol Buffers
- **Request/response matching:** multiplexed over a single bidirectional stream using per-connection request IDs
- **Server identity distribution:** `ServerInfo` protobuf struct containing the TLS public key fingerprint
- **Future consideration:** grpc-web lacks bidirectional stream support, so a browser-based wallet may require protojson over WebSocket

### Request Multiplexing

Both `client` and `operator` connections support multiple in-flight requests over one gRPC bidi stream.

- Every request carries a monotonically increasing request ID
- Every normal response echoes the request ID it corresponds to
- Out-of-band server messages omit the response ID entirely
- The server rejects already-seen request IDs at the transport adapter boundary before business logic sees the message

This keeps request correlation entirely in transport/client connection code while leaving actor and domain handlers unaware of request IDs.

---

## EVM Policy Engine

### Overview

The EVM engine classifies incoming transactions, enforces grant constraints, and records executions. It is the sole path through which a wallet key is used for signing.

The central abstraction is the `Policy` trait. Each implementation handles one semantic transaction category and owns its own database tables for grant storage and transaction logging.

### Transaction Evaluation Flow

`Engine::evaluate_transaction` runs the following steps in order:

1. **Classify** — Each registered policy's `analyze(context)` inspects the transaction fields (`chain`, `to`, `value`, `calldata`). The first one returning `Some(meaning)` wins. If none match, the transaction is rejected as `UnsupportedTransactionType`.
2. **Find grant** — `Policy::try_find_grant` queries for a non-revoked grant covering this wallet, client, chain, and target address.
3. **Check shared constraints** — `check_shared_constraints` runs in the engine before any policy-specific logic. It enforces the validity window, gas fee caps, and transaction count rate limit (see below).
4. **Evaluate** — `Policy::evaluate` checks the decoded meaning against the grant's policy-specific constraints and returns any violations.
5. **Record** — If `RunKind::Execution` and there are no violations, the engine writes to `evm_transaction_log` and calls `Policy::record_transaction` for any policy-specific logging (e.g., token transfer volume).

The detailed branch structure is shown below:

```mermaid
flowchart TD
    A[SDK Client sends sign transaction request] --> B[Server resolves wallet]
    B --> C{Wallet exists?}

    C -- No --> Z1[Return wallet not found error]
    C -- Yes --> D[Check SDK client wallet visibility]

    D --> E{Wallet visible to SDK client?}
    E -- No --> F[Start wallet visibility voting flow]
    F --> G{Vote approved?}
    G -- No --> Z2[Return wallet access denied error]
    G -- Yes --> H[Persist wallet visibility]
    E -- Yes --> I[Classify transaction meaning]
    H --> I

    I --> J{Meaning supported?}
    J -- No --> Z3[Return unsupported transaction error]
    J -- Yes --> K[Find matching grant]

    K --> L{Grant exists?}
    L -- Yes --> M[Check grant limits]
    L -- No --> N[Start execution or grant voting flow]

    N --> O{Operator decision}
    O -- Reject --> Z4[Return no matching grant error]
    O -- Allow once --> M
    O -- Create grant --> P[Create grant with user-selected limits]
    P --> Q[Persist grant]
    Q --> M

    M --> R{Limits exceeded?}
    R -- Yes --> Z5[Return evaluation error]
    R -- No --> S[Record transaction in logs]
    S --> T[Produce signature]
    T --> U[Return signature to SDK client]

    note1[Limit checks include volume, count, and gas constraints.]
    note2[Grant lookup depends on classified meaning, such as ether transfer or token transfer.]

    K -. uses .-> note2
    M -. checks .-> note1
```

### Policy Trait

| Method | Purpose |
|---|---|
| `analyze` | Pure — classifies a transaction into a typed `Meaning`, or `None` if this policy doesn't apply |
| `evaluate` | Checks the `Meaning` against a `Grant`; returns a list of `EvalViolation`s |
| `create_grant` | Inserts policy-specific rows; returns the specific grant ID |
| `try_find_grant` | Finds a matching non-revoked grant for the given `EvalContext` |
| `find_all_grants` | Returns all non-revoked grants (used for listing) |
| `record_transaction` | Persists policy-specific data after execution |

`analyze` and `evaluate` are intentionally separate: classification is pure and cheap, while evaluation may involve DB queries (e.g., fetching past transfer volume).

### Registered Policies

**EtherTransfer** — plain ETH transfers (empty calldata)

- Grant requires: allowlist of recipient addresses + one volumetric rate limit (max ETH over a time window)
- Violations: recipient not in allowlist, cumulative ETH volume exceeded

**TokenTransfer** — ERC-20 `transfer(address,uint256)` calls

- Recognised by ABI-decoding the `transfer(address,uint256)` selector against a static registry of known token contracts (`arbiter_tokens_registry`)
- Grant requires: token contract address, optional recipient restriction, zero or more volumetric rate limits
- Violations: recipient mismatch, any volumetric limit exceeded

### Grant Model

Every grant has two layers:

- **Shared (`evm_basic_grant`)** — wallet, chain, validity period, gas fee caps, transaction count rate limit. One row per grant regardless of type.
- **Specific** — policy-owned tables (`evm_ether_transfer_grant`, `evm_token_transfer_grant`) holding type-specific configuration.

`find_all_grants` uses a `#[diesel::auto_type]` base join between the specific and shared tables, then batch-loads related rows (targets, volume limits) in two additional queries to avoid N+1.

The engine exposes `list_all_grants` which collects across all policy types into `Vec<Grant<SpecificGrant>>` via a blanket `From<Grant<S>> for Grant<SpecificGrant>` conversion.

### Shared Constraints (enforced by the engine)

These are checked centrally in `check_shared_constraints` before policy evaluation:

| Constraint | Fields | Behaviour |
|---|---|---|
| Validity window | `valid_from`, `valid_until` | Emits `InvalidTime` if current time is outside the range |
| Gas fee cap | `max_gas_fee_per_gas`, `max_priority_fee_per_gas` | Emits `GasLimitExceeded` if either cap is breached |
| Tx count rate limit | `rate_limit` (`count` + `window`) | Counts rows in `evm_transaction_log` within the window; emits `RateLimitExceeded` if at or above the limit |

---

### Known Limitations

- **Only EIP-1559 transactions are supported.** Legacy and EIP-2930 types are rejected outright.
- **No opaque-calldata (unknown contract) grant type.** The architecture describes a category for unrecognised contracts, but no policy implements it yet. Any transaction that is not a plain ETH transfer or a known ERC-20 transfer is unconditionally rejected.
- **Token registry is static.** Tokens are recognised only if they appear in the hard-coded `arbiter_tokens_registry` crate. There is no mechanism to register additional contracts at runtime.

---

## Memory Protection

The unsealed root key must be held in a hardened memory cell resistant to dumps, page swaps, and hibernation.

- **Current:** A dedicated memory-protection abstraction is in place, with `memsafe` used behind that abstraction today
- **Planned:** Additional backends can be introduced behind the same abstraction, including a custom implementation based on `mlock` (Unix) and `VirtualProtect` (Windows)
