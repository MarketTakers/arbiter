import 'dart:math' as math;

import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:arbiter/providers/evm.dart';
import 'package:arbiter/providers/evm_grants.dart';
import 'package:arbiter/router.gr.dart';
import 'package:arbiter/widgets/bottom_popup.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class EvmGrantsScreen extends ConsumerWidget {
  const EvmGrantsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final grantsAsync = ref.watch(evmGrantsProvider);
    final grantsState = grantsAsync.asData?.value;
    final wallets = ref.watch(evmProvider).asData?.value ?? const <WalletEntry>[];
    final revokeMutation = ref.watch(revokeEvmGrantMutation);
    final isConnected =
        ref.watch(connectionManagerProvider).asData?.value != null;

    Future<void> refresh() async {
      try {
        await ref.read(evmGrantsProvider.notifier).refresh();
      } catch (error) {
        if (!context.mounted) {
          return;
        }
        _showMessage(context, _formatError(error));
      }
    }

    Future<void> revokeGrant(GrantEntry grant) async {
      try {
        await executeRevokeEvmGrant(ref, grantId: grant.id);
        if (context.mounted) {
          Navigator.of(context).pop();
          _showMessage(context, 'Grant revoked.');
        }
      } catch (error) {
        if (!context.mounted) {
          return;
        }
        _showMessage(context, _formatError(error));
      }
    }

    Future<void> openCreate() async {
      await context.router.push(const CreateEvmGrantRoute());
    }

    final content = switch (grantsAsync) {
      AsyncLoading() when grantsState == null => const _GrantStatePanel(
        icon: Icons.hourglass_top,
        title: 'Loading grants',
        body: 'Pulling EVM grants and wallet context from Arbiter.',
        busy: true,
      ),
      AsyncError(:final error) => _GrantStatePanel(
        icon: Icons.sync_problem,
        title: 'Grant registry unavailable',
        body: _formatError(error),
        actionLabel: 'Retry',
        onAction: refresh,
      ),
      _ when !isConnected => _GrantStatePanel(
        icon: Icons.portable_wifi_off,
        title: 'No active server connection',
        body: 'Reconnect to Arbiter to inspect or create EVM grants.',
        actionLabel: 'Refresh',
        onAction: refresh,
      ),
      _ when grantsState == null => const SizedBox.shrink(),
      _ when grantsState.grants.isEmpty => _GrantStatePanel(
        icon: Icons.rule_folder_outlined,
        title: 'No grants yet',
        body:
            'Create the first grant to authorize scoped transaction signing for a client.',
        actionLabel: 'Create grant',
        onAction: openCreate,
      ),
      _ => _GrantGrid(
        state: grantsState,
        wallets: wallets,
        onGrantTap: (grant) {
          return showBottomPopup<void>(
            context: context,
            builder: (popupContext) => _GrantDetailSheet(
              grant: grant,
              wallets: wallets,
              isRevoking: revokeMutation is MutationPending<void>,
              onRevoke: () => revokeGrant(grant),
            ),
          );
        },
      ),
    };

    return Scaffold(
      body: SafeArea(
        child: RefreshIndicator.adaptive(
          color: _GrantPalette.ink,
          backgroundColor: Colors.white,
          onRefresh: refresh,
          child: ListView(
            physics: const BouncingScrollPhysics(
              parent: AlwaysScrollableScrollPhysics(),
            ),
            padding: EdgeInsets.fromLTRB(2.4.w, 2.4.h, 2.4.w, 3.2.h),
            children: [
              _GrantHeader(
                state: grantsState,
                isRefreshing: grantsAsync.isLoading,
                onRefresh: refresh,
                onCreate: openCreate,
                onToggleShowRevoked: (value) {
                  ref.read(evmGrantsProvider.notifier).toggleShowRevoked(value);
                  if (value) {
                    _showMessage(
                      context,
                      'Revoked grant history is waiting on backend support. Active grants only for now.',
                    );
                  }
                },
              ),
              if (grantsState?.showRevoked == true)
                Padding(
                  padding: EdgeInsets.only(top: 1.4.h),
                  child: const _RevokedSupportBanner(),
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

class _GrantPalette {
  static const ink = Color(0xFF17324A);
  static const sea = Color(0xFF0F766E);
  static const gold = Color(0xFFE19A2A);
  static const coral = Color(0xFFE46B56);
  static const mist = Color(0xFFF7F8FB);
  static const line = Color(0x1A17324A);
}

class _GrantHeader extends StatelessWidget {
  const _GrantHeader({
    required this.state,
    required this.isRefreshing,
    required this.onRefresh,
    required this.onCreate,
    required this.onToggleShowRevoked,
  });

  final EvmGrantsState? state;
  final bool isRefreshing;
  final Future<void> Function() onRefresh;
  final Future<void> Function() onCreate;
  final ValueChanged<bool> onToggleShowRevoked;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final menuValue = state?.showRevoked ?? false;

    return Container(
      padding: EdgeInsets.symmetric(horizontal: 1.8.w, vertical: 1.4.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(22),
        gradient: const LinearGradient(
          colors: [Color(0xFFF6F8FC), Color(0xFFFDF7EF)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        border: Border.all(color: _GrantPalette.line),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'EVM Grants',
                  style: theme.textTheme.titleLarge?.copyWith(
                    color: _GrantPalette.ink,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                SizedBox(height: 0.4.h),
                Text(
                  'Browse active permissions, inspect policy details, and create new grants.',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: _GrantPalette.ink.withValues(alpha: 0.70),
                    height: 1.35,
                  ),
                ),
              ],
            ),
          ),
          IconButton.filledTonal(
            tooltip: 'Filters',
            onPressed: () async {
              final button = context.findRenderObject() as RenderBox?;
              final overlay =
                  Overlay.of(context).context.findRenderObject() as RenderBox?;
              if (button == null || overlay == null) {
                return;
              }
              final selected = await showMenu<bool>(
                context: context,
                position: RelativeRect.fromRect(
                  Rect.fromPoints(
                    button.localToGlobal(Offset.zero, ancestor: overlay),
                    button.localToGlobal(
                      button.size.bottomRight(Offset.zero),
                      ancestor: overlay,
                    ),
                  ),
                  Offset.zero & overlay.size,
                ),
                items: [
                  CheckedPopupMenuItem<bool>(
                    value: !menuValue,
                    checked: menuValue,
                    child: Text(
                      menuValue ? 'Hide revoked grants' : 'Show revoked grants',
                    ),
                  ),
                ],
              );
              if (selected != null) {
                onToggleShowRevoked(selected);
              }
            },
            icon: const Icon(Icons.filter_list_rounded),
          ),
          SizedBox(width: 0.8.w),
          OutlinedButton.icon(
            onPressed: isRefreshing ? null : () => onRefresh(),
            icon: isRefreshing
                ? SizedBox(
                    width: 1.8.h,
                    height: 1.8.h,
                    child: const CircularProgressIndicator(strokeWidth: 2.2),
                  )
                : const Icon(Icons.refresh_rounded, size: 18),
            label: const Text('Refresh'),
          ),
          SizedBox(width: 0.8.w),
          FilledButton.icon(
            onPressed: () => onCreate(),
            style: FilledButton.styleFrom(
              backgroundColor: _GrantPalette.ink,
              foregroundColor: Colors.white,
            ),
            icon: const Icon(Icons.add_rounded, size: 18),
            label: const Text('Create'),
          ),
        ],
      ),
    );
  }
}

class _RevokedSupportBanner extends StatelessWidget {
  const _RevokedSupportBanner();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(1.6.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(18),
        color: _GrantPalette.gold.withValues(alpha: 0.12),
        border: Border.all(color: _GrantPalette.gold.withValues(alpha: 0.28)),
      ),
      child: Row(
        children: [
          Icon(Icons.info_outline_rounded, color: _GrantPalette.gold),
          SizedBox(width: 1.2.w),
          Expanded(
            child: Text(
              'Revoked grant history is not exposed by the current backend yet. This screen still shows active grants only.',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                color: _GrantPalette.ink.withValues(alpha: 0.78),
                height: 1.35,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _GrantGrid extends StatelessWidget {
  const _GrantGrid({
    required this.state,
    required this.wallets,
    required this.onGrantTap,
  });

  final EvmGrantsState state;
  final List<WalletEntry> wallets;
  final Future<void> Function(GrantEntry grant) onGrantTap;

  @override
  Widget build(BuildContext context) {
    final grantsByWallet = <int, List<GrantEntry>>{};
    for (final grant in state.grants) {
      grantsByWallet.putIfAbsent(grant.shared.walletId, () => []).add(grant);
    }

    final walletIds = grantsByWallet.keys.toList()..sort();

    return Column(
      children: [
        for (final walletId in walletIds)
          Padding(
            padding: EdgeInsets.only(bottom: 1.8.h),
            child: _WalletGrantSection(
              walletId: walletId,
              walletAddress: _addressForWalletId(wallets, walletId),
              grants: grantsByWallet[walletId]!,
              onGrantTap: onGrantTap,
            ),
          ),
      ],
    );
  }
}

class _WalletGrantSection extends StatelessWidget {
  const _WalletGrantSection({
    required this.walletId,
    required this.walletAddress,
    required this.grants,
    required this.onGrantTap,
  });

  final int walletId;
  final List<int>? walletAddress;
  final List<GrantEntry> grants;
  final Future<void> Function(GrantEntry grant) onGrantTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: _GrantPalette.mist,
        border: Border.all(color: _GrantPalette.line),
      ),
      padding: EdgeInsets.all(2.h),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Wallet ${walletId.toString().padLeft(2, '0')}',
            style: theme.textTheme.titleMedium?.copyWith(
              color: _GrantPalette.ink,
              fontWeight: FontWeight.w800,
            ),
          ),
          SizedBox(height: 0.4.h),
          Text(
            walletAddress == null
                ? 'Wallet address unavailable in the current API.'
                : _hexAddress(walletAddress!),
            style: theme.textTheme.bodySmall?.copyWith(
              color: _GrantPalette.ink.withValues(alpha: 0.70),
            ),
          ),
          SizedBox(height: 1.8.h),
          LayoutBuilder(
            builder: (context, constraints) {
              final maxWidth = constraints.maxWidth;
              final cardWidth = maxWidth >= 900
                  ? (maxWidth - 2.w * 2) / 3
                  : maxWidth >= 620
                  ? (maxWidth - 2.w) / 2
                  : maxWidth;
              return Wrap(
                spacing: 1.4.w,
                runSpacing: 1.4.h,
                children: [
                  for (final grant in grants)
                    SizedBox(
                      width: math.max(280, cardWidth),
                      child: _GrantCardRouter(
                        grant: grant,
                        walletAddress: walletAddress,
                        onTap: () => onGrantTap(grant),
                      ),
                    ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }
}

class _GrantCardRouter extends StatelessWidget {
  const _GrantCardRouter({
    required this.grant,
    required this.walletAddress,
    required this.onTap,
  });

  final GrantEntry grant;
  final List<int>? walletAddress;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return switch (grant.specific.whichGrant()) {
      SpecificGrant_Grant.etherTransfer => _EtherGrantCard(
        grant: grant,
        walletAddress: walletAddress,
        onTap: onTap,
      ),
      SpecificGrant_Grant.tokenTransfer => _TokenGrantCard(
        grant: grant,
        walletAddress: walletAddress,
        onTap: onTap,
      ),
      _ => _UnsupportedGrantCard(grant: grant, onTap: onTap),
    };
  }
}

class _GrantCardFrame extends StatelessWidget {
  const _GrantCardFrame({
    required this.icon,
    required this.iconColor,
    required this.title,
    required this.subtitle,
    required this.chips,
    required this.onTap,
  });

  final IconData icon;
  final Color iconColor;
  final String title;
  final String subtitle;
  final List<String> chips;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Material(
      color: Colors.white,
      borderRadius: BorderRadius.circular(22),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(22),
        child: Ink(
          padding: EdgeInsets.all(2.h),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(22),
            border: Border.all(color: _GrantPalette.line),
            boxShadow: [
              BoxShadow(
                color: _GrantPalette.ink.withValues(alpha: 0.05),
                blurRadius: 24,
                offset: const Offset(0, 12),
              ),
            ],
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Align(
                child: Container(
                  width: 6.2.h,
                  height: 6.2.h,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: iconColor.withValues(alpha: 0.14),
                  ),
                  child: Icon(icon, color: iconColor, size: 3.h),
                ),
              ),
              SizedBox(height: 1.6.h),
              Text(
                title,
                style: theme.textTheme.titleMedium?.copyWith(
                  color: _GrantPalette.ink,
                  fontWeight: FontWeight.w800,
                ),
              ),
              SizedBox(height: 0.5.h),
              Text(
                subtitle,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: _GrantPalette.ink.withValues(alpha: 0.72),
                  height: 1.35,
                ),
              ),
              SizedBox(height: 1.4.h),
              Wrap(
                spacing: 0.8.w,
                runSpacing: 0.8.h,
                children: [
                  for (final chip in chips) _GrantChip(label: chip),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _EtherGrantCard extends StatelessWidget {
  const _EtherGrantCard({
    required this.grant,
    required this.walletAddress,
    required this.onTap,
  });

  final GrantEntry grant;
  final List<int>? walletAddress;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final settings = grant.specific.etherTransfer;
    final targets = settings.targets.length;
    final subtitle = targets == 0
        ? 'ETH transfers with a shared limit profile.'
        : '$targets target${targets == 1 ? '' : 's'} authorized.';

    return _GrantCardFrame(
      icon: Icons.bolt_rounded,
      iconColor: _GrantPalette.gold,
      title: 'Ether Transfer',
      subtitle: subtitle,
      chips: [
        'Client ${grant.clientId}',
        'Wallet ${grant.shared.walletId}',
        if (walletAddress != null) _shortAddress(walletAddress!),
      ],
      onTap: onTap,
    );
  }
}

class _TokenGrantCard extends StatelessWidget {
  const _TokenGrantCard({
    required this.grant,
    required this.walletAddress,
    required this.onTap,
  });

  final GrantEntry grant;
  final List<int>? walletAddress;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final settings = grant.specific.tokenTransfer;

    return _GrantCardFrame(
      icon: Icons.token_rounded,
      iconColor: _GrantPalette.sea,
      title: 'Token Transfer',
      subtitle:
          'Contract ${_shortAddress(settings.tokenContract)} with ${settings.volumeLimits.length} volume rule${settings.volumeLimits.length == 1 ? '' : 's'}.',
      chips: [
        'Client ${grant.clientId}',
        'Wallet ${grant.shared.walletId}',
        if (walletAddress != null) _shortAddress(walletAddress!),
      ],
      onTap: onTap,
    );
  }
}

class _UnsupportedGrantCard extends StatelessWidget {
  const _UnsupportedGrantCard({required this.grant, required this.onTap});

  final GrantEntry grant;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return _GrantCardFrame(
      icon: Icons.help_outline_rounded,
      iconColor: _GrantPalette.coral,
      title: 'Unsupported Grant',
      subtitle: 'This grant type cannot be rendered in the current useragent.',
      chips: ['Grant ${grant.id}'],
      onTap: onTap,
    );
  }
}

class _GrantChip extends StatelessWidget {
  const _GrantChip({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.symmetric(horizontal: 1.1.w, vertical: 0.7.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(999),
        color: _GrantPalette.ink.withValues(alpha: 0.06),
      ),
      child: Text(
        label,
        style: Theme.of(context).textTheme.labelMedium?.copyWith(
          color: _GrantPalette.ink.withValues(alpha: 0.76),
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class _GrantDetailSheet extends StatelessWidget {
  const _GrantDetailSheet({
    required this.grant,
    required this.wallets,
    required this.isRevoking,
    required this.onRevoke,
  });

  final GrantEntry grant;
  final List<WalletEntry> wallets;
  final bool isRevoking;
  final Future<void> Function() onRevoke;

  @override
  Widget build(BuildContext context) {
    final walletAddress = _addressForWalletId(wallets, grant.shared.walletId);

    return Container(
      width: 100.w,
      constraints: BoxConstraints(maxWidth: 760),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(28),
        color: Colors.white,
      ),
      padding: EdgeInsets.all(2.2.h),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  'Grant #${grant.id}',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    color: _GrantPalette.ink,
                    fontWeight: FontWeight.w800,
                  ),
                ),
              ),
              IconButton(
                onPressed: () => Navigator.of(context).pop(),
                icon: const Icon(Icons.close_rounded),
              ),
            ],
          ),
          SizedBox(height: 1.2.h),
          Wrap(
            spacing: 1.w,
            runSpacing: 0.8.h,
            children: [
              _GrantChip(label: 'Client ${grant.clientId}'),
              _GrantChip(label: 'Wallet ${grant.shared.walletId}'),
              if (walletAddress != null) _GrantChip(label: _shortAddress(walletAddress)),
            ],
          ),
          SizedBox(height: 2.h),
          _SectionTitle(title: 'Shared policy'),
          _FieldSummary(label: 'Chain ID', value: grant.shared.chainId.toString()),
          _FieldSummary(
            label: 'Validity',
            value: _validitySummary(grant.shared),
          ),
          _FieldSummary(
            label: 'Gas fee cap',
            value: _optionalBigInt(grant.shared.maxGasFeePerGas),
          ),
          _FieldSummary(
            label: 'Priority fee cap',
            value: _optionalBigInt(grant.shared.maxPriorityFeePerGas),
          ),
          _FieldSummary(
            label: 'Tx count limit',
            value: grant.shared.hasRateLimit()
                ? '${grant.shared.rateLimit.count} tx / ${grant.shared.rateLimit.windowSecs}s'
                : 'Not set',
          ),
          SizedBox(height: 1.8.h),
          _SectionTitle(title: 'Grant-specific settings'),
          switch (grant.specific.whichGrant()) {
            SpecificGrant_Grant.etherTransfer => _EtherGrantDetails(
              settings: grant.specific.etherTransfer,
            ),
            SpecificGrant_Grant.tokenTransfer => _TokenGrantDetails(
              settings: grant.specific.tokenTransfer,
            ),
            _ => const Text('Unsupported grant type'),
          },
          SizedBox(height: 2.2.h),
          Align(
            alignment: Alignment.centerRight,
            child: FilledButton.icon(
              onPressed: isRevoking ? null : () => onRevoke(),
              style: FilledButton.styleFrom(
                backgroundColor: _GrantPalette.coral,
                foregroundColor: Colors.white,
              ),
              icon: isRevoking
                  ? SizedBox(
                      width: 1.8.h,
                      height: 1.8.h,
                      child: const CircularProgressIndicator(strokeWidth: 2.2),
                    )
                  : const Icon(Icons.block_rounded),
              label: Text(isRevoking ? 'Revoking...' : 'Revoke grant'),
            ),
          ),
        ],
      ),
    );
  }
}

class _EtherGrantDetails extends StatelessWidget {
  const _EtherGrantDetails({required this.settings});

  final EtherTransferSettings settings;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _FieldSummary(
          label: 'Targets',
          value: settings.targets.isEmpty
              ? 'No explicit target restriction'
              : settings.targets.map(_hexAddress).join(', '),
        ),
        _FieldSummary(
          label: 'Volume limit',
          value: settings.hasLimit()
              ? '${_optionalBigInt(settings.limit.maxVolume)} / ${settings.limit.windowSecs}s'
              : 'Not set',
        ),
      ],
    );
  }
}

class _TokenGrantDetails extends StatelessWidget {
  const _TokenGrantDetails({required this.settings});

  final TokenTransferSettings settings;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _FieldSummary(
          label: 'Token contract',
          value: _hexAddress(settings.tokenContract),
        ),
        _FieldSummary(
          label: 'Recipient',
          value: settings.hasTarget() ? _hexAddress(settings.target) : 'Any recipient',
        ),
        _FieldSummary(
          label: 'Volume rules',
          value: settings.volumeLimits.isEmpty
              ? 'Not set'
              : settings.volumeLimits
                    .map(
                      (limit) =>
                          '${_optionalBigInt(limit.maxVolume)} / ${limit.windowSecs}s',
                    )
                    .join('\n'),
        ),
      ],
    );
  }
}

class _FieldSummary extends StatelessWidget {
  const _FieldSummary({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(bottom: 1.h),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: Theme.of(context).textTheme.labelLarge?.copyWith(
              color: _GrantPalette.ink.withValues(alpha: 0.60),
              fontWeight: FontWeight.w800,
            ),
          ),
          SizedBox(height: 0.3.h),
          Text(
            value,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: _GrantPalette.ink,
              height: 1.4,
            ),
          ),
        ],
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle({required this.title});

  final String title;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(bottom: 1.1.h),
      child: Text(
        title,
        style: Theme.of(context).textTheme.titleMedium?.copyWith(
          color: _GrantPalette.ink,
          fontWeight: FontWeight.w800,
        ),
      ),
    );
  }
}

class _GrantStatePanel extends StatelessWidget {
  const _GrantStatePanel({
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
    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: _GrantPalette.mist,
        border: Border.all(color: _GrantPalette.line),
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
              Icon(icon, size: 34, color: _GrantPalette.coral),
            SizedBox(height: 1.8.h),
            Text(
              title,
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                color: _GrantPalette.ink,
                fontWeight: FontWeight.w800,
              ),
            ),
            SizedBox(height: 1.h),
            Text(
              body,
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                color: _GrantPalette.ink.withValues(alpha: 0.72),
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

List<int>? _addressForWalletId(List<WalletEntry> wallets, int walletId) {
  final index = walletId - 1;
  if (index < 0 || index >= wallets.length) {
    return null;
  }
  return wallets[index].address;
}

String _shortAddress(List<int> bytes) {
  final value = _hexAddress(bytes);
  if (value.length <= 14) {
    return value;
  }
  return '${value.substring(0, 8)}...${value.substring(value.length - 4)}';
}

String _hexAddress(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '0x$hex';
}

String _optionalBigInt(List<int> bytes) {
  if (bytes.isEmpty) {
    return 'Not set';
  }
  return _bytesToBigInt(bytes).toString();
}

String _validitySummary(SharedSettings shared) {
  final from = shared.hasValidFrom()
      ? DateTime.fromMillisecondsSinceEpoch(
          shared.validFrom.seconds.toInt() * 1000,
          isUtc: true,
        ).toLocal().toString()
      : 'Immediate';
  final until = shared.hasValidUntil()
      ? DateTime.fromMillisecondsSinceEpoch(
          shared.validUntil.seconds.toInt() * 1000,
          isUtc: true,
        ).toLocal().toString()
      : 'Open-ended';
  return '$from -> $until';
}

BigInt _bytesToBigInt(List<int> bytes) {
  return bytes.fold<BigInt>(
    BigInt.zero,
    (value, byte) => (value << 8) | BigInt.from(byte),
  );
}

void _showMessage(BuildContext context, String message) {
  if (!context.mounted) {
    return;
  }
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
  );
}

String _formatError(Object error) {
  final message = error.toString();
  if (message.startsWith('Exception: ')) {
    return message.substring('Exception: '.length);
  }
  return message;
}
