# Grant Grid View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an "EVM Grants" dashboard tab that displays all grants as enriched cards (type, chain, wallet address, client name) with per-card revoke support.

**Architecture:** A new `walletAccessListProvider` fetches wallet accesses with their DB row IDs. The screen (`grants.dart`) watches only `evmGrantsProvider` for top-level state. Each `GrantCard` widget (its own file) watches enrichment providers (`walletAccessListProvider`, `evmProvider`, `sdkClientsProvider`) and the revoke mutation directly — keeping rebuilds scoped to the card. The screen is registered as a dashboard tab in `AdaptiveScaffold`.

**Tech Stack:** Flutter, Riverpod (`riverpod_annotation` + `build_runner` codegen), `sizer` (adaptive sizing), `auto_route`, Protocol Buffers (Dart), `Palette` design tokens.

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `operator/lib/theme/palette.dart` | Modify | Add `Palette.token` (indigo accent for token-transfer cards) |
| `operator/lib/features/connection/evm/wallet_access.dart` | Modify | Add `listAllWalletAccesses()` function |
| `operator/lib/providers/sdk_clients/wallet_access_list.dart` | Create | `WalletAccessListProvider` — fetches full wallet access list with IDs |
| `operator/lib/screens/dashboard/evm/grants/widgets/grant_card.dart` | Create | `GrantCard` widget — watches enrichment providers + revoke mutation; one card per grant |
| `operator/lib/screens/dashboard/evm/grants/grants.dart` | Create | `EvmGrantsScreen` — watches `evmGrantsProvider`; handles loading/error/empty/data states; renders `GrantCard` list |
| `operator/lib/router.dart` | Modify | Register `EvmGrantsRoute` in dashboard children |
| `operator/lib/screens/dashboard.dart` | Modify | Add Grants entry to `routes` list and `NavigationDestination` list |

---

## Task 1: Add `Palette.token`

**Files:**
- Modify: `operator/lib/theme/palette.dart`

- [ ] **Step 1: Add the color**

Replace the contents of `operator/lib/theme/palette.dart` with:

```dart
import 'package:flutter/material.dart';

class Palette {
  static const ink = Color(0xFF15263C);
  static const coral = Color(0xFFE26254);
  static const cream = Color(0xFFFFFAF4);
  static const line = Color(0x1A15263C);
  static const token = Color(0xFF5C6BC0);
}
```

- [ ] **Step 2: Verify**

```sh
cd operator && flutter analyze lib/theme/palette.dart
```

Expected: no issues.

- [ ] **Step 3: Commit**

```sh
jj describe -m "feat(theme): add Palette.token for token-transfer grant cards"
jj new
```

---

## Task 2: Add `listAllWalletAccesses` feature function

**Files:**
- Modify: `operator/lib/features/connection/evm/wallet_access.dart`

`readClientWalletAccess` (existing) filters the list to one client's wallet IDs and returns `Set<int>`. This new function returns the complete unfiltered list with row IDs so the grant cards can resolve wallet_access_id → wallet + client.

- [ ] **Step 1: Append function**

Add at the bottom of `operator/lib/features/connection/evm/wallet_access.dart`:

```dart
Future<List<SdkClientWalletAccess>> listAllWalletAccesses(
  Connection connection,
) async {
  final response = await connection.ask(
    OperatorRequest(listWalletAccess: Empty()),
  );
  if (!response.hasListWalletAccessResponse()) {
    throw Exception(
      'Expected list wallet access response, got ${response.whichPayload()}',
    );
  }
  return response.listWalletAccessResponse.accesses.toList(growable: false);
}
```

Each returned `SdkClientWalletAccess` has:
- `.id` — the `evm_wallet_access` row ID (same value as `wallet_access_id` in a `GrantEntry`)
- `.access.walletId` — the EVM wallet DB ID
- `.access.sdkClientId` — the SDK client DB ID

- [ ] **Step 2: Verify**

```sh
cd operator && flutter analyze lib/features/connection/evm/wallet_access.dart
```

Expected: no issues.

- [ ] **Step 3: Commit**

```sh
jj describe -m "feat(evm): add listAllWalletAccesses feature function"
jj new
```

---

## Task 3: Create `WalletAccessListProvider`

**Files:**
- Create: `operator/lib/providers/sdk_clients/wallet_access_list.dart`
- Generated: `operator/lib/providers/sdk_clients/wallet_access_list.g.dart`

Mirrors the structure of `EvmGrants` in `providers/evm/evm_grants.dart` — class-based `@riverpod` with a `refresh()` method.

- [ ] **Step 1: Write the provider**

Create `operator/lib/providers/sdk_clients/wallet_access_list.dart`:

```dart
import 'package:arbiter/features/connection/evm/wallet_access.dart';
import 'package:arbiter/proto/operator.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:mtcore/markettakers.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'wallet_access_list.g.dart';

@riverpod
class WalletAccessList extends _$WalletAccessList {
  @override
  Future<List<SdkClientWalletAccess>?> build() async {
    final connection = await ref.watch(connectionManagerProvider.future);
    if (connection == null) {
      return null;
    }

    try {
      return await listAllWalletAccesses(connection);
    } catch (e, st) {
      talker.handle(e, st);
      rethrow;
    }
  }

  Future<void> refresh() async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      state = const AsyncData(null);
      return;
    }

    state = const AsyncLoading();
    state = await AsyncValue.guard(() => listAllWalletAccesses(connection));
  }
}
```

- [ ] **Step 2: Run code generation**

```sh
cd operator && dart run build_runner build --delete-conflicting-outputs
```

Expected: `operator/lib/providers/sdk_clients/wallet_access_list.g.dart` created. No errors.

- [ ] **Step 3: Verify**

```sh
cd operator && flutter analyze lib/providers/sdk_clients/
```

Expected: no issues.

- [ ] **Step 4: Commit**

```sh
jj describe -m "feat(providers): add WalletAccessListProvider"
jj new
```

---

## Task 4: Create `GrantCard` widget

**Files:**
- Create: `operator/lib/screens/dashboard/evm/grants/widgets/grant_card.dart`

This widget owns all per-card logic: enrichment lookups, revoke action, and rebuild scope. The screen only passes it a `GrantEntry` — the card fetches everything else itself.

**Key types:**
- `GrantEntry` (from `proto/evm.pb.dart`): `.id`, `.shared.walletAccessId`, `.shared.chainId`, `.specific.whichGrant()`
- `SpecificGrant_Grant.etherTransfer` / `.tokenTransfer` — enum values for the oneof
- `SdkClientWalletAccess` (from `proto/operator.pb.dart`): `.id`, `.access.walletId`, `.access.sdkClientId`
- `WalletEntry` (from `proto/evm.pb.dart`): `.id`, `.address` (List<int>)
- `SdkClientEntry` (from `proto/operator.pb.dart`): `.id`, `.info.name`
- `revokeEvmGrantMutation` — `Mutation<void>` (global; all revoke buttons disable together while any revoke is in flight)
- `executeRevokeEvmGrant(ref, grantId: int)` — `Future<void>`

- [ ] **Step 1: Write the widget**

Create `operator/lib/screens/dashboard/evm/grants/widgets/grant_card.dart`:

```dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/operator.pb.dart';
import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/providers/evm/evm_grants.dart';
import 'package:arbiter/providers/sdk_clients/list.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access_list.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

String _shortAddress(List<int> bytes) {
  final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  return '0x${hex.substring(0, 6)}...${hex.substring(hex.length - 4)}';
}

String _formatError(Object error) {
  final message = error.toString();
  if (message.startsWith('Exception: ')) {
    return message.substring('Exception: '.length);
  }
  return message;
}

class GrantCard extends ConsumerWidget {
  const GrantCard({super.key, required this.grant});

  final GrantEntry grant;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Enrichment lookups — each watch scopes rebuilds to this card only
    final walletAccesses =
        ref.watch(walletAccessListProvider).asData?.value ?? const [];
    final wallets = ref.watch(evmProvider).asData?.value ?? const [];
    final clients = ref.watch(sdkClientsProvider).asData?.value ?? const [];
    final revoking = ref.watch(revokeEvmGrantMutation) is MutationPending;

    final isEther =
        grant.specific.whichGrant() == SpecificGrant_Grant.etherTransfer;
    final accent = isEther ? Palette.coral : Palette.token;
    final typeLabel = isEther ? 'Ether' : 'Token';
    final theme = Theme.of(context);
    final muted = Palette.ink.withValues(alpha: 0.62);

    // Resolve wallet_access_id → wallet address + client name
    final accessById = <int, SdkClientWalletAccess>{
      for (final a in walletAccesses) a.id: a,
    };
    final walletById = <int, WalletEntry>{
      for (final w in wallets) w.id: w,
    };
    final clientNameById = <int, String>{
      for (final c in clients) c.id: c.info.name,
    };

    final accessId = grant.shared.walletAccessId;
    final access = accessById[accessId];
    final wallet = access != null ? walletById[access.access.walletId] : null;

    final walletLabel = wallet != null
        ? _shortAddress(wallet.address)
        : 'Access #$accessId';

    final clientLabel = () {
      if (access == null) return '';
      final name = clientNameById[access.access.sdkClientId] ?? '';
      return name.isEmpty ? 'Client #${access.access.sdkClientId}' : name;
    }();

    void showError(String message) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
      );
    }

    Future<void> revoke() async {
      try {
        await executeRevokeEvmGrant(ref, grantId: grant.id);
      } catch (e) {
        showError(_formatError(e));
      }
    }

    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: Palette.cream.withValues(alpha: 0.92),
        border: Border.all(color: Palette.line),
      ),
      child: IntrinsicHeight(
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Accent strip
            Container(
              width: 0.8.w,
              decoration: BoxDecoration(
                color: accent,
                borderRadius: const BorderRadius.horizontal(
                  left: Radius.circular(24),
                ),
              ),
            ),
            // Card body
            Expanded(
              child: Padding(
                padding: EdgeInsets.symmetric(
                  horizontal: 1.6.w,
                  vertical: 1.4.h,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Row 1: type badge · chain · spacer · revoke button
                    Row(
                      children: [
                        Container(
                          padding: EdgeInsets.symmetric(
                            horizontal: 1.w,
                            vertical: 0.4.h,
                          ),
                          decoration: BoxDecoration(
                            color: accent.withValues(alpha: 0.15),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Text(
                            typeLabel,
                            style: theme.textTheme.labelSmall?.copyWith(
                              color: accent,
                              fontWeight: FontWeight.w800,
                            ),
                          ),
                        ),
                        SizedBox(width: 1.w),
                        Container(
                          padding: EdgeInsets.symmetric(
                            horizontal: 1.w,
                            vertical: 0.4.h,
                          ),
                          decoration: BoxDecoration(
                            color: Palette.ink.withValues(alpha: 0.06),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Text(
                            'Chain ${grant.shared.chainId}',
                            style: theme.textTheme.labelSmall?.copyWith(
                              color: muted,
                              fontWeight: FontWeight.w700,
                            ),
                          ),
                        ),
                        const Spacer(),
                        if (revoking)
                          SizedBox(
                            width: 1.8.h,
                            height: 1.8.h,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              color: Palette.coral,
                            ),
                          )
                        else
                          OutlinedButton.icon(
                            onPressed: revoke,
                            style: OutlinedButton.styleFrom(
                              foregroundColor: Palette.coral,
                              side: BorderSide(
                                color: Palette.coral.withValues(alpha: 0.4),
                              ),
                              padding: EdgeInsets.symmetric(
                                horizontal: 1.w,
                                vertical: 0.6.h,
                              ),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(10),
                              ),
                            ),
                            icon: const Icon(Icons.block_rounded, size: 16),
                            label: const Text('Revoke'),
                          ),
                      ],
                    ),
                    SizedBox(height: 0.8.h),
                    // Row 2: wallet address · client name
                    Row(
                      children: [
                        Text(
                          walletLabel,
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: Palette.ink,
                            fontFamily: 'monospace',
                          ),
                        ),
                        Padding(
                          padding: EdgeInsets.symmetric(horizontal: 0.8.w),
                          child: Text(
                            '·',
                            style: theme.textTheme.bodySmall
                                ?.copyWith(color: muted),
                          ),
                        ),
                        Expanded(
                          child: Text(
                            clientLabel,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: theme.textTheme.bodySmall
                                ?.copyWith(color: muted),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
```

- [ ] **Step 2: Verify**

```sh
cd operator && flutter analyze lib/screens/dashboard/evm/grants/widgets/grant_card.dart
```

Expected: no issues.

- [ ] **Step 3: Commit**

```sh
jj describe -m "feat(grants): add GrantCard widget with self-contained enrichment"
jj new
```

---

## Task 5: Create `EvmGrantsScreen`

**Files:**
- Create: `operator/lib/screens/dashboard/evm/grants/grants.dart`

The screen watches only `evmGrantsProvider` for top-level state (loading / error / no connection / empty / data). When there is data it renders a list of `GrantCard` widgets — each card manages its own enrichment subscriptions.

- [ ] **Step 1: Write the screen**

Create `operator/lib/screens/dashboard/evm/grants/grants.dart`:

```dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/evm/evm_grants.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access_list.dart';
import 'package:arbiter/router.gr.dart';
import 'package:arbiter/screens/dashboard/evm/grants/widgets/grant_card.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:arbiter/widgets/page_header.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

String _formatError(Object error) {
  final message = error.toString();
  if (message.startsWith('Exception: ')) {
    return message.substring('Exception: '.length);
  }
  return message;
}

// ─── State panel ──────────────────────────────────────────────────────────────

class _StatePanel extends StatelessWidget {
  const _StatePanel({
    required this.icon,
    required this.title,
    required this.body,
    this.actionLabel,
    this.onAction,
    this.busy = false,
  });

  final IconData icon;
  final String title;
  final String body;
  final String? actionLabel;
  final Future<void> Function()? onAction;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: Palette.cream.withValues(alpha: 0.92),
        border: Border.all(color: Palette.line),
      ),
      child: Padding(
        padding: EdgeInsets.all(2.8.h),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (busy)
              SizedBox(
                width: 2.8.h,
                height: 2.8.h,
                child: const CircularProgressIndicator(strokeWidth: 2.5),
              )
            else
              Icon(icon, size: 34, color: Palette.coral),
            SizedBox(height: 1.8.h),
            Text(
              title,
              style: theme.textTheme.headlineSmall?.copyWith(
                color: Palette.ink,
                fontWeight: FontWeight.w800,
              ),
            ),
            SizedBox(height: 1.h),
            Text(
              body,
              style: theme.textTheme.bodyLarge?.copyWith(
                color: Palette.ink.withValues(alpha: 0.72),
                height: 1.5,
              ),
            ),
            if (actionLabel != null && onAction != null) ...[
              SizedBox(height: 2.h),
              OutlinedButton.icon(
                onPressed: () => onAction!(),
                icon: const Icon(Icons.refresh),
                label: Text(actionLabel!),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

// ─── Grant list ───────────────────────────────────────────────────────────────

class _GrantList extends StatelessWidget {
  const _GrantList({required this.grants});

  final List<GrantEntry> grants;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        for (var i = 0; i < grants.length; i++)
          Padding(
            padding: EdgeInsets.only(
              bottom: i == grants.length - 1 ? 0 : 1.8.h,
            ),
            child: GrantCard(grant: grants[i]),
          ),
      ],
    );
  }
}

// ─── Screen ───────────────────────────────────────────────────────────────────

@RoutePage()
class EvmGrantsScreen extends ConsumerWidget {
  const EvmGrantsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Screen watches only the grant list for top-level state decisions
    final grantsAsync = ref.watch(evmGrantsProvider);

    Future<void> refresh() async {
      await Future.wait([
        ref.read(evmGrantsProvider.notifier).refresh(),
        ref.read(walletAccessListProvider.notifier).refresh(),
      ]);
    }

    void showMessage(String message) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
      );
    }

    Future<void> safeRefresh() async {
      try {
        await refresh();
      } catch (e) {
        showMessage(_formatError(e));
      }
    }

    final grantsState = grantsAsync.asData?.value;
    final grants = grantsState?.grants;

    final content = switch (grantsAsync) {
      AsyncLoading() when grantsState == null => const _StatePanel(
        icon: Icons.hourglass_top,
        title: 'Loading grants',
        body: 'Pulling grant registry from Arbiter.',
        busy: true,
      ),
      AsyncError(:final error) => _StatePanel(
        icon: Icons.sync_problem,
        title: 'Grant registry unavailable',
        body: _formatError(error),
        actionLabel: 'Retry',
        onAction: safeRefresh,
      ),
      AsyncData(:final value) when value == null => _StatePanel(
        icon: Icons.portable_wifi_off,
        title: 'No active server connection',
        body: 'Reconnect to Arbiter to list EVM grants.',
        actionLabel: 'Refresh',
        onAction: safeRefresh,
      ),
      _ when grants != null && grants.isEmpty => _StatePanel(
        icon: Icons.policy_outlined,
        title: 'No grants yet',
        body: 'Create a grant to allow SDK clients to sign transactions.',
        actionLabel: 'Create grant',
        onAction: () => context.router.push(const CreateEvmGrantRoute()),
      ),
      _ => _GrantList(grants: grants ?? const []),
    };

    return Scaffold(
      body: SafeArea(
        child: RefreshIndicator.adaptive(
          color: Palette.ink,
          backgroundColor: Colors.white,
          onRefresh: safeRefresh,
          child: ListView(
            physics: const BouncingScrollPhysics(
              parent: AlwaysScrollableScrollPhysics(),
            ),
            padding: EdgeInsets.fromLTRB(2.4.w, 2.4.h, 2.4.w, 3.2.h),
            children: [
              PageHeader(
                title: 'EVM Grants',
                isBusy: grantsAsync.isLoading,
                actions: [
                  FilledButton.icon(
                    onPressed: () =>
                        context.router.push(const CreateEvmGrantRoute()),
                    icon: const Icon(Icons.add_rounded),
                    label: const Text('Create grant'),
                  ),
                  SizedBox(width: 1.w),
                  OutlinedButton.icon(
                    onPressed: safeRefresh,
                    style: OutlinedButton.styleFrom(
                      foregroundColor: Palette.ink,
                      side: BorderSide(color: Palette.line),
                      padding: EdgeInsets.symmetric(
                        horizontal: 1.4.w,
                        vertical: 1.2.h,
                      ),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(14),
                      ),
                    ),
                    icon: const Icon(Icons.refresh, size: 18),
                    label: const Text('Refresh'),
                  ),
                ],
              ),
              SizedBox(height: 1.8.h),
              content,
            ],
          ),
        ),
      ),
    );
  }
}
```

- [ ] **Step 2: Verify**

```sh
cd operator && flutter analyze lib/screens/dashboard/evm/grants/
```

Expected: no issues.

- [ ] **Step 3: Commit**

```sh
jj describe -m "feat(grants): add EvmGrantsScreen"
jj new
```

---

## Task 6: Wire router and dashboard tab

**Files:**
- Modify: `operator/lib/router.dart`
- Modify: `operator/lib/screens/dashboard.dart`
- Regenerated: `operator/lib/router.gr.dart`

- [ ] **Step 1: Add route to `router.dart`**

Replace the contents of `operator/lib/router.dart` with:

```dart
import 'package:auto_route/auto_route.dart';

import 'router.gr.dart';

@AutoRouterConfig(generateForDir: ['lib/screens'])
class Router extends RootStackRouter {
  @override
  List<AutoRoute> get routes => [
    AutoRoute(page: Bootstrap.page, path: '/bootstrap', initial: true),
    AutoRoute(page: ServerInfoSetupRoute.page, path: '/server-info'),
    AutoRoute(page: ServerConnectionRoute.page, path: '/server-connection'),
    AutoRoute(page: VaultSetupRoute.page, path: '/vault'),
    AutoRoute(page: ClientDetailsRoute.page, path: '/clients/:clientId'),
    AutoRoute(page: CreateEvmGrantRoute.page, path: '/evm-grants/create'),

    AutoRoute(
      page: DashboardRouter.page,
      path: '/dashboard',
      children: [
        AutoRoute(page: EvmRoute.page, path: 'evm'),
        AutoRoute(page: ClientsRoute.page, path: 'clients'),
        AutoRoute(page: EvmGrantsRoute.page, path: 'grants'),
        AutoRoute(page: AboutRoute.page, path: 'about'),
      ],
    ),
  ];
}
```

- [ ] **Step 2: Update `dashboard.dart`**

In `operator/lib/screens/dashboard.dart`, replace the `routes` constant:

```dart
final routes = [
  const EvmRoute(),
  const ClientsRoute(),
  const EvmGrantsRoute(),
  const AboutRoute(),
];
```

And replace the `destinations` list inside `AdaptiveScaffold`:

```dart
destinations: const [
  NavigationDestination(
    icon: Icon(Icons.account_balance_wallet_outlined),
    selectedIcon: Icon(Icons.account_balance_wallet),
    label: 'Wallets',
  ),
  NavigationDestination(
    icon: Icon(Icons.devices_other_outlined),
    selectedIcon: Icon(Icons.devices_other),
    label: 'Clients',
  ),
  NavigationDestination(
    icon: Icon(Icons.policy_outlined),
    selectedIcon: Icon(Icons.policy),
    label: 'Grants',
  ),
  NavigationDestination(
    icon: Icon(Icons.info_outline),
    selectedIcon: Icon(Icons.info),
    label: 'About',
  ),
],
```

- [ ] **Step 3: Regenerate router**

```sh
cd operator && dart run build_runner build --delete-conflicting-outputs
```

Expected: `lib/router.gr.dart` updated, `EvmGrantsRoute` now available, no errors.

- [ ] **Step 4: Full project verify**

```sh
cd operator && flutter analyze
```

Expected: no issues.

- [ ] **Step 5: Commit**

```sh
jj describe -m "feat(nav): add Grants dashboard tab"
jj new
```
