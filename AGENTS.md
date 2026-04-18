# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Arbiter is a **permissioned signing service** for cryptocurrency wallets. It consists of:
- **`server/`** — Rust gRPC daemon that holds encrypted keys and enforces policies
- **`useragent/`** — Flutter desktop app (macOS/Windows) with a Rust backend via Rinf
- **`protobufs/`** — Protocol Buffer definitions shared between server and client

The vault never exposes key material; it only produces signatures when requests satisfy configured policies.

## Toolchain Setup

Tools are managed via [mise](https://mise.jdx.dev/). Install all required tools:
```sh
mise install
```

Key versions: Rust 1.93.0 (with clippy), Flutter 3.38.9-stable, protoc 29.6, diesel_cli 2.3.6 (sqlite).

## Server (Rust workspace at `server/`)

### Crates

| Crate | Purpose |
|---|---|
| `arbiter-proto` | Generated gRPC stubs + protobuf types; compiled from `protobufs/*.proto` via `tonic-prost-build` |
| `arbiter-server` | Main daemon — actors, DB, EVM policy engine, gRPC service implementation |
| `arbiter-useragent` | Rust client library for the user agent side of the gRPC protocol |
| `arbiter-client` | Rust client library for SDK clients |

### Common Commands

```sh
cd server

# Build
cargo build

# Run the server daemon
cargo run -p arbiter-server

# Run all tests (preferred over cargo test)
cargo nextest run

# Run a single test
cargo nextest run <test_name>

# Lint
cargo clippy

# Security audit
cargo audit

# Check unused dependencies
cargo shear

# Run snapshot tests and update snapshots
cargo insta review
```

### Architecture

The server is actor-based using the **kameo** crate. All long-lived state lives in `GlobalActors`:

- **`Bootstrapper`** — Manages the one-time bootstrap token written to `~/.arbiter/bootstrap_token` on first run.
- **`Vault`** — Holds the encrypted root key and manages the Sealed/Unsealed vault state machine. On unseal, decrypts the root key into a `memsafe` hardened memory cell.
- **`FlowCoordinator`** — Coordinates cross-connection flow between user agents and SDK clients.
- **`EvmActor`** — Handles EVM transaction policy enforcement and signing.

Per-connection actors live under `actors/user_agent/` and `actors/client/`, each with `auth` (challenge-response authentication) and `session` (post-auth operations) sub-modules.

**Database:** SQLite via `diesel-async` + `bb8` connection pool. Schema managed by embedded Diesel migrations in `crates/arbiter-server/migrations/`. DB file lives at `~/.arbiter/arbiter.sqlite`. Tests use a temp-file DB via `db::create_test_pool()`.

**Cryptography:**
- Authentication: ed25519 (challenge-response, nonce-tracked per peer)
- Encryption at rest: XChaCha20-Poly1305 (versioned via `scheme` field for transparent migration on unseal)
- Password KDF: Argon2
- Unseal transport: X25519 ephemeral key exchange
- TLS: self-signed certificate (aws-lc-rs backend), fingerprint distributed via `ArbiterUrl`

**Protocol:** gRPC with Protocol Buffers. The `ArbiterUrl` type encodes host, port, CA cert, and bootstrap token into a single shareable string (printed to console on first run).

### Proto Regeneration

When `.proto` files in `protobufs/` change, rebuild to regenerate:
```sh
cd server && cargo build -p arbiter-proto
```

### Database Migrations

```sh
# Create a new migration
diesel migration generate <name> --migration-dir crates/arbiter-server/migrations

# Run migrations manually (server also runs them on startup)
diesel migration run --migration-dir crates/arbiter-server/migrations
```

### Code Conventions

**`#[must_use]` Attribute:**
Apply the `#[must_use]` attribute to return types of functions where the return value is critical and should not be accidentally ignored. This is commonly used for:

- Methods that return `bool` indicating success/failure or validation state
- Any function where ignoring the return value indicates a logic error

Do not apply `#[must_use]` redundantly to items (types or functions) that are already annotated with `#[must_use]`.

Example:

```rust
#[must_use]
pub fn verify(&self, nonce: i32, context: &[u8], signature: &Signature) -> bool {
    // verification logic
}
```

This forces callers to either use the return value or explicitly ignore it with `let _ = ...;`, preventing silent failures.

## User Agent (Flutter + Rinf at `useragent/`)

The Flutter app uses [Rinf](https://rinf.cunarist.org) to call Rust code. The Rust logic lives in `useragent/native/hub/` as a separate crate that uses `arbiter-useragent` for the gRPC client.

Communication between Dart and Rust uses typed **signals** defined in `useragent/native/hub/src/signals/`. After modifying signal structs, regenerate Dart bindings:

```sh
cd useragent && rinf gen
```

### Common Commands

```sh
cd useragent

# Run the app (macOS or Windows)
flutter run

# Regenerate Rust↔Dart signal bindings
rinf gen

# Analyze Dart code
flutter analyze
```

The Rinf Rust entry point is `useragent/native/hub/src/lib.rs`. It spawns actors defined in `useragent/native/hub/src/actors/` which handle Dart↔server communication via signals.
