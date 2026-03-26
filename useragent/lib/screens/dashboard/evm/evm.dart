import 'dart:math' as math;

import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/widgets/page_header.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class EvmScreen extends HookConsumerWidget {
  const EvmScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final walletsAsync = ref.watch(evmProvider);
    final isCreating = useState(false);

    final wallets = walletsAsync.asData?.value;
    final loadedWallets = wallets ?? const <WalletEntry>[];
    final isConnected =
        ref.watch(connectionManagerProvider).asData?.value != null;

    void showMessage(String message) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
      );
    }

    Future<void> refreshWallets() async {
      try {
        await ref.read(evmProvider.notifier).refreshWallets();
      } catch (error) {
        showMessage(_formatError(error));
      }
    }

    Future<void> createWallet() async {
      if (isCreating.value) {
        return;
      }

      isCreating.value = true;
      try {
        await ref.read(evmProvider.notifier).createWallet();
        showMessage('Wallet created.');
      } catch (error) {
        showMessage(_formatError(error));
      } finally {
        isCreating.value = false;
      }
    }

    final content = switch (walletsAsync) {
      AsyncLoading() when wallets == null => const _StatePanel(
        icon: Icons.hourglass_top,
        title: 'Loading wallets',
        body: 'Pulling wallet registry from Arbiter.',
        busy: true,
      ),
      AsyncError(:final error) => _StatePanel(
        icon: Icons.sync_problem,
        title: 'Wallet registry unavailable',
        body: _formatError(error),
        actionLabel: 'Retry',
        onAction: refreshWallets,
      ),
      _ when !isConnected => _StatePanel(
        icon: Icons.portable_wifi_off,
        title: 'No active server connection',
        body: 'Reconnect to Arbiter to list or create EVM wallets.',
        actionLabel: 'Refresh',
        onAction: refreshWallets,
      ),
      _ when loadedWallets.isEmpty => _StatePanel(
        icon: Icons.account_balance_wallet_outlined,
        title: 'No wallets yet',
        body:
            'Create the first vault-backed wallet to start building your EVM registry.',
        actionLabel: isCreating.value ? 'Creating...' : 'Create wallet',
        onAction: isCreating.value ? null : createWallet,
      ),
      _ => _WalletTable(wallets: loadedWallets),
    };

    return Scaffold(
      body: SafeArea(
        child: RefreshIndicator.adaptive(
          color: Palette.ink,
          backgroundColor: Colors.white,
          onRefresh: refreshWallets,
          child: ListView(
            physics: const BouncingScrollPhysics(
              parent: AlwaysScrollableScrollPhysics(),
            ),
            padding: EdgeInsets.fromLTRB(2.4.w, 2.4.h, 2.4.w, 3.2.h),
            children: [
              PageHeader(
                title: 'EVM Wallet Vault',
                isBusy: walletsAsync.isLoading,
                actions: [
                  FilledButton.icon(
                    onPressed: isCreating.value ? null : () => createWallet(),
                    style: FilledButton.styleFrom(
                      backgroundColor: Palette.ink,
                      foregroundColor: Colors.white,
                      padding: EdgeInsets.symmetric(
                        horizontal: 1.4.w,
                        vertical: 1.2.h,
                      ),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(14),
                      ),
                    ),
                    icon: isCreating.value
                        ? SizedBox(
                            width: 1.6.h,
                            height: 1.6.h,
                            child: CircularProgressIndicator(strokeWidth: 2.2),
                          )
                        : const Icon(Icons.add_circle_outline, size: 18),
                    label: Text(isCreating.value ? 'Creating...' : 'Create'),
                  ),
                  SizedBox(width: 1.w),
                  OutlinedButton.icon(
                    onPressed: () => refreshWallets(),
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

double get _accentStripWidth => 0.8.w;
double get _cellHorizontalPadding => 1.8.w;
double get _walletColumnWidth => 18.w;
double get _columnGap => 1.8.w;
double get _tableMinWidth => 72.w;

class _WalletTable extends StatelessWidget {
  const _WalletTable({required this.wallets});

  final List<WalletEntry> wallets;

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
        padding: EdgeInsets.all(2.h),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final tableWidth = math.max(_tableMinWidth, constraints.maxWidth);

            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Managed wallets',
                  style: theme.textTheme.titleLarge?.copyWith(
                    color: Palette.ink,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                SizedBox(height: 0.6.h),
                Text(
                  'Every address here is generated and held by Arbiter.',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: Palette.ink.withValues(alpha: 0.70),
                    height: 1.4,
                  ),
                ),
                SizedBox(height: 1.6.h),
                SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: SizedBox(
                    width: tableWidth,
                    child: Column(
                      children: [
                        const _WalletTableHeader(),
                        SizedBox(height: 1.h),
                        for (var i = 0; i < wallets.length; i++)
                          Padding(
                            padding: EdgeInsets.only(
                              bottom: i == wallets.length - 1 ? 0 : 1.h,
                            ),
                            child: _WalletTableRow(
                              wallet: wallets[i],
                              index: i,
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

class _WalletTableHeader extends StatelessWidget {
  const _WalletTableHeader();

  @override
  Widget build(BuildContext context) {
    final style = Theme.of(context).textTheme.labelLarge?.copyWith(
      color: Palette.ink.withValues(alpha: 0.72),
      fontWeight: FontWeight.w800,
      letterSpacing: 0.3,
    );

    return Container(
      padding: EdgeInsets.symmetric(vertical: 1.4.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(16),
        color: Palette.ink.withValues(alpha: 0.04),
      ),
      child: Row(
        children: [
          SizedBox(width: _accentStripWidth + _cellHorizontalPadding),
          SizedBox(
            width: _walletColumnWidth,
            child: Text('Wallet', style: style),
          ),
          SizedBox(width: _columnGap),
          Expanded(child: Text('Address', style: style)),
          SizedBox(width: _cellHorizontalPadding),
        ],
      ),
    );
  }
}

class _WalletTableRow extends StatelessWidget {
  const _WalletTableRow({required this.wallet, required this.index});

  final WalletEntry wallet;
  final int index;

  @override
  Widget build(BuildContext context) {
    final accent = _accentColor(wallet.address);
    final address = _hexAddress(wallet.address);
    final rowHeight = 5.h;
    final walletStyle = Theme.of(
      context,
    ).textTheme.bodyLarge?.copyWith(color: Palette.ink);
    final addressStyle = Theme.of(
      context,
    ).textTheme.bodyMedium?.copyWith(color: Palette.ink);

    return Container(
      height: rowHeight,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(18),
        color: accent.withValues(alpha: 0.10),
        border: Border.all(color: accent.withValues(alpha: 0.28)),
      ),
      child: Row(
        children: [
          Container(
            width: _accentStripWidth,
            height: rowHeight,
            decoration: BoxDecoration(
              color: accent,
              borderRadius: const BorderRadius.horizontal(
                left: Radius.circular(18),
              ),
            ),
          ),
          Expanded(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: _cellHorizontalPadding),
              child: Row(
                children: [
                  SizedBox(
                    width: _walletColumnWidth,
                    child: Row(
                      children: [
                        Container(
                          width: 1.2.h,
                          height: 1.2.h,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: accent,
                          ),
                        ),
                        SizedBox(width: 1.w),
                        Text(
                          'Wallet ${(index + 1).toString().padLeft(2, '0')}',
                          style: walletStyle,
                        ),
                      ],
                    ),
                  ),
                  SizedBox(width: _columnGap),
                  Expanded(
                    child: Text(
                      address,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: addressStyle,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

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
                child: CircularProgressIndicator(strokeWidth: 2.5),
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

String _hexAddress(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '0x$hex';
}

Color _accentColor(List<int> bytes) {
  final seed = bytes.fold<int>(0, (value, byte) => value + byte);
  final hue = (seed * 17) % 360;
  return HSLColor.fromAHSL(1, hue.toDouble(), 0.68, 0.54).toColor();
}

String _formatError(Object error) {
  final message = error.toString();
  if (message.startsWith('Exception: ')) {
    return message.substring('Exception: '.length);
  }
  return message;
}
