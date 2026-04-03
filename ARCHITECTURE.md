# Arbiter

Arbiter is a permissioned signing service for cryptocurrency wallets. It runs as a background service on the user's machine with an optional client application for vault management.

**Core principle:** The vault NEVER exposes key material. It only produces signatures when a request satisfies the configured policies.
---

## 1. Peer Types

Arbiter distinguishes two kinds of peers:

- **User Agent** — A client application used by the owner to manage the vault (create wallets, approve SDK clients, configure policies).
- **SDK Client** — A consumer of signing capabilities, typically an automation tool. In the future, this could include a browser-based wallet.

---

## 2. Authentication

### 2.1 Challenge-Response

All peers authenticate via public-key cryptography using a challenge-response protocol:

1. The peer sends its public key and requests a challenge.
2. The server looks up the key in its database. If found, it increments the nonce and returns a challenge (replay-attack protection).
3. The peer signs the challenge with its private key and sends the signature back.
4. The server verifies the signature:
   - **Pass:** The connection is considered authenticated.
   - **Fail:** The server closes the connection.

### 2.2 User Agent Bootstrap

On first run — when no User Agents are registered — the server generates a one-time bootstrap token. It is made available in two ways:

- **Local setup:** Written to `~/.arbiter/bootstrap_token` for automatic discovery by a co-located User Agent.
- **Remote setup:** Printed to the server's console output.

The first User Agent must present this token alongside the standard challenge-response to complete registration.

### 2.3 SDK Client Registration

There is no bootstrap mechanism for SDK clients. They must be explicitly approved by an already-registered User Agent.

---

## 3. Multi-Operator Governance

When more than one User Agent is registered, the vault is treated as having multiple operators. In that mode, sensitive actions are governed by voting rather than by a single operator decision.

### 3.1 Voting Rules

Voting is based on the total number of registered operators:

- **1 operator:** no vote is needed; the single operator decides directly.
- **2 operators:** full consensus is required; both operators must approve.
- **3 or more operators:** quorum is `floor(N / 2) + 1`.

Examples:

- **3 operators:** 2 approvals required
- **4 operators:** 3 approvals required

### 3.2 Actions Requiring a Vote

In multi-operator mode, a successful vote is required for:

- approving new SDK clients
- granting an SDK client visibility to a wallet
- approving a one-off transaction
- approving creation of a persistent grant
- approving server updates
- updating Shamir secret-sharing parameters

### 3.3 Special Rule for Key Rotation

Key rotation always requires full quorum, regardless of the normal voting threshold.

This is stricter than ordinary governance actions because rotating the root key requires every operator to participate in coordinated share refresh/update steps. The root key itself is not redistributed directly, but each operator's share material must be changed consistently.

### 3.4 Root Key Custody

When the vault has multiple operators, the vault root key is protected using Shamir secret sharing.

This ensures that root-key recovery and governance-sensitive changes are aligned with the multi-operator model rather than delegated to a single operator-held secret.

---

## 4. Server Identity

The server proves its identity using TLS with a self-signed certificate. The TLS private key is generated on first run and is long-term; no rotation mechanism exists yet due to the complexity of multi-peer coordination.

Peers verify the server by its **public key fingerprint**:

- **User Agent (local):** Receives the fingerprint automatically through the bootstrap token.
- **User Agent (remote) / SDK Client:** Must receive the fingerprint out-of-band.

> A streamlined setup mechanism using a single connection string is planned but not yet implemented.

---

## 5. Key Management

### 5.1 Key Hierarchy

There are three layers of keys:

| Key | Encrypts | Encrypted by |
|---|---|---|
| **User key** (password) | Root key | — (derived from user input) |
| **Root key** | Wallet keys | User key |
| **Wallet keys** | — (used for signing) | Root key |

This layered design enables:

- **Password rotation** without re-encrypting every wallet key (only the root key is re-encrypted).
- **Root key rotation** without requiring the user to change their password.

### 5.2 Encryption at Rest

The database stores everything in encrypted form using symmetric AEAD. The encryption scheme is versioned to support transparent migration — when the vault unseals, Arbiter automatically re-encrypts any entries that are behind the current scheme version. See [IMPLEMENTATION.md](IMPLEMENTATION.md) for the specific scheme and versioning mechanism.

---

## 6. Vault Lifecycle

### 6.1 Sealed State

On boot, the root key is encrypted and the server cannot perform any signing operations. This state is called **Sealed**.

### 6.2 Unseal Flow

To transition to the **Unsealed** state, a User Agent must provide the password:

1. The User Agent initiates an unseal request.
2. The server generates a one-time key pair and returns the public key.
3. The User Agent encrypts the user's password with this one-time public key and sends the ciphertext to the server.
4. The server decrypts and verifies the password:
   - **Success:** The root key is decrypted and placed into a hardened memory cell. The server transitions to `Unsealed`. Any entries pending encryption scheme migration are re-encrypted.
   - **Failure:** The server returns an error indicating the password is incorrect.

### 6.3 Memory Protection

Once unsealed, the root key must be protected in memory against:

- Memory dumps
- Page swaps to disk
- Hibernation files

See [IMPLEMENTATION.md](IMPLEMENTATION.md) for the current and planned memory protection approaches.

---

## 7. Permission Engine

### 7.1 Fundamental Rules

- SDK clients have **no access by default**.
- Access is granted **explicitly** by a User Agent.
- Grants are scoped to **specific wallets** and governed by **policies**.

Each blockchain requires its own policy system due to differences in static transaction analysis. Currently, only EVM is supported; Solana support is planned.

Arbiter is also responsible for ensuring that **transaction nonces are never reused**.

### 7.2 EVM Policies

Every EVM grant is scoped to a specific **wallet** and **chain ID**.

#### 7.2.0 Transaction Signing Sequence

The high-level interaction order is:

```mermaid
sequenceDiagram
    autonumber
    actor SDK as SDK Client
    participant Server
    participant UA as User Agent

    SDK->>Server: SignTransactionRequest
    Server->>Server: Resolve wallet and wallet visibility
    alt Visibility approval required
        Server->>UA: Ask for wallet visibility approval
        UA-->>Server: Vote result
    end
    Server->>Server: Evaluate transaction
    Server->>Server: Load grant and limits context
    alt Grant approval required
        Server->>UA: Ask for execution / grant approval
        UA-->>Server: Vote result
        opt Create persistent grant
            Server->>Server: Create and store grant
        end
        Server->>Server: Retry evaluation
    end
    critical Final authorization path
        Server->>Server: Check limits and record execution
        Server-->>Server: Signature or evaluation error
    end
    Server-->>SDK: Signature or error
```

#### 7.2.1 Transaction Sub-Grants

Arbiter maintains an ever-expanding database of known contracts and their ABIs. Based on contract knowledge, transaction requests fall into three categories:

**1. Known contract (ABI available)**

The transaction can be decoded and presented with semantic meaning. For example: *"Client X wants to transfer Y USDT to address Z."*

Available restrictions:
- Volume limits (e.g., "no more than 10,000 tokens ever")
- Rate limits (e.g., "no more than 100 tokens per hour")

**2. Unknown contract (no ABI)**

The transaction cannot be decoded, so its effects are opaque — it could do anything, including draining all tokens. The user is warned, and if approved, access is granted to all interactions with the contract (matched by the `to` field).

Available restrictions:
- Transaction count limits (e.g., "no more than 100 transactions ever")
- Rate limits (e.g., "no more than 5 transactions per hour")

**3. Plain ether transfer (no calldata)**

These transactions have no `calldata` and therefore cannot interact with contracts. They can be subject to the same volume and rate restrictions as above.

#### 7.2.2 Global Limits

In addition to sub-grant-specific restrictions, the following limits can be applied across all grant types:

- **Gas limit** — Maximum gas per transaction.
- **Time-window restrictions** — e.g., signing allowed only 08:00–20:00 on Mondays and Thursdays.
