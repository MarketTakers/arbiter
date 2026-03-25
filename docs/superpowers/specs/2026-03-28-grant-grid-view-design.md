# Grant Grid View — Design Spec

**Date:** 2026-03-28

## Overview

Add a "Grants" dashboard tab to the Flutter user-agent app that displays all EVM grants as a card-based grid. Each card shows a compact summary (type, chain, wallet address, client name) with a revoke action. The tab integrates into the existing `AdaptiveScaffold` navigation alongside Wallets, Clients, and About.

## Scope

- New `walletAccessListProvider` for fetching wallet access entries with their DB row IDs
- New `EvmGrantsScreen` as a dashboard tab
- Grant card widget with enriched display (type, chain, wallet, client)
- Revoke action wired to existing `executeRevokeEvmGrant` mutation
- Dashboard tab bar and router updated
- New token-transfer accent color added to `Palette`

**Out of scope:** Fixing grant creation (separate task).

---

## Data Layer

### `walletAccessListProvider`

**File:** `useragent/lib/providers/sdk_clients/wallet_access_list.dart`

- `@riverpod` class, watches `connectionManagerProvider.future`
- Returns `List<SdkClientWalletAccess>?` (null when not connected)
- Each entry: `.id` (wallet_access_id), `.access.walletId`, `.access.sdkClientId`
- Exposes a `refresh()` method following the same pattern as `EvmGrants.refresh()`

### Enrichment at render time (Approach A)

The `EvmGrantsScreen` watches four providers:
1. `evmGrantsProvider` — the grant list
2. `walletAccessListProvider` — to resolve wallet_access_id → (wallet_id, sdk_client_id)
3. `evmProvider` — to resolve wallet_id → wallet address
4. `sdkClientsProvider` — to resolve sdk_client_id → client name

All lookups are in-memory Maps built inside the build method; no extra model class needed.

Fallbacks:
- Wallet address not found → `"Access #N"` where N is the wallet_access_id
- Client name not found → `"Client #N"` where N is the sdk_client_id

---

## Route Structure

```
/dashboard
  /evm          ← existing (Wallets tab)
  /clients      ← existing (Clients tab)
  /grants       ← NEW (Grants tab)
  /about        ← existing

/evm-grants/create  ← existing push route (unchanged)
```

### Changes to `router.dart`

Add inside dashboard children:
```dart
AutoRoute(page: EvmGrantsRoute.page, path: 'grants'),
```

### Changes to `dashboard.dart`

Add to `routes` list:
```dart
const EvmGrantsRoute()
```

Add `NavigationDestination`:
```dart
NavigationDestination(
  icon: Icon(Icons.policy_outlined),
  selectedIcon: Icon(Icons.policy),
  label: 'Grants',
),
```

---

## Screen: `EvmGrantsScreen`

**File:** `useragent/lib/screens/dashboard/evm/grants/grants.dart`

```
Scaffold
└─ SafeArea
   └─ RefreshIndicator.adaptive (refreshes evmGrantsProvider + walletAccessListProvider)
      └─ ListView (BouncingScrollPhysics + AlwaysScrollableScrollPhysics)
         ├─ PageHeader
         │    title: 'EVM Grants'
         │    isBusy: evmGrantsProvider.isLoading
         │    actions: [CreateGrantButton, RefreshButton]
         ├─ SizedBox(height: 1.8.h)
         └─ <content>
```

### State handling

Matches the pattern from `EvmScreen` and `ClientsScreen`:

| State | Display |
|---|---|
| Loading (no data yet) | `_StatePanel` with spinner, "Loading grants" |
| Error | `_StatePanel` with coral icon, error message, Retry button |
| No connection | `_StatePanel`, "No active server connection" |
| Empty list | `_StatePanel`, "No grants yet", with Create Grant shortcut |
| Data | Column of `_GrantCard` widgets |

### Header actions

**CreateGrantButton:** `FilledButton.icon` with `Icons.add_rounded`, pushes `CreateEvmGrantRoute()` via `context.router.push(...)`.

**RefreshButton:** `OutlinedButton.icon` with `Icons.refresh`, calls `ref.read(evmGrantsProvider.notifier).refresh()`.

---

## Grant Card: `_GrantCard`

**Layout:**

```
Container (rounded 24, Palette.cream bg, Palette.line border)
└─ IntrinsicHeight > Row
   ├─ Accent strip (0.8.w wide, full height, rounded left)
   └─ Padding > Column
        ├─ Row 1: TypeBadge + ChainChip + Spacer + RevokeButton
        └─ Row 2: WalletText + "·" + ClientText
```

**Accent color by grant type:**
- Ether transfer → `Palette.coral`
- Token transfer → `Palette.token` (new entry in `Palette` — indigo, e.g. `Color(0xFF5C6BC0)`)

**TypeBadge:** Small pill container with accent color background at 15% opacity, accent-colored text. Label: `'Ether'` or `'Token'`.

**ChainChip:** Small container: `'Chain ${grant.shared.chainId}'`, muted ink color.

**WalletText:** Short hex address (`0xabc...def`) from wallet lookup, `bodySmall`, monospace font family.

**ClientText:** Client name from `sdkClientsProvider` lookup, or fallback string. `bodySmall`, muted ink.

**RevokeButton:**
- `OutlinedButton` with `Icons.block_rounded` icon, label `'Revoke'`
- `foregroundColor: Palette.coral`, `side: BorderSide(color: Palette.coral.withValues(alpha: 0.4))`
- Disabled (replaced with `CircularProgressIndicator`) while `revokeEvmGrantMutation` is pending — note: this is a single global mutation, so all revoke buttons disable while any revoke is in flight
- On press: calls `executeRevokeEvmGrant(ref, grantId: grant.id)`; shows `SnackBar` on error

---

## Adaptive Sizing

All sizing uses `sizer` units (`1.h`, `1.w`, etc.). No hardcoded pixel values.

---

## Files to Create / Modify

| File | Action |
|---|---|
| `lib/theme/palette.dart` | Modify — add `Palette.token` color |
| `lib/providers/sdk_clients/wallet_access_list.dart` | Create |
| `lib/screens/dashboard/evm/grants/grants.dart` | Create |
| `lib/router.dart` | Modify — add grants route to dashboard children |
| `lib/screens/dashboard.dart` | Modify — add tab to routes list and NavigationDestinations |
