import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/screens/dashboard/evm/wallets/header.dart';
import 'package:arbiter/screens/dashboard/evm/wallets/table.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/widgets/page_header.dart';
import 'package:arbiter/widgets/state_panel.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class EvmScreen extends HookConsumerWidget {
  const EvmScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final evm = ref.watch(evmProvider);

    final wallets = evm.asData?.value;
    final loadedWallets = wallets ?? const <WalletEntry>[];

    void showMessage(String message) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
      );
    }

    Future<void> refreshWallets() async {
      try {
        await ref.read(evmProvider.notifier).refreshWallets();
      } catch (e) {
        showMessage('Failed to refresh wallets: ${_formatError(e)}');
      }
    }

    final content = switch (evm) {
      AsyncLoading() when wallets == null => const StatePanel(
        icon: Icons.hourglass_top,
        title: 'Loading wallets',
        body: 'Pulling wallet registry from Arbiter.',
        busy: true,
      ),
      AsyncError(:final error) => StatePanel(
        icon: Icons.sync_problem,
        title: 'Wallet registry unavailable',
        body: _formatError(error),
        actionLabel: 'Retry',
        onAction: refreshWallets,
      ),
      AsyncData(:final value) when value == null => StatePanel(
        icon: Icons.portable_wifi_off,
        title: 'No active server connection',
        body: 'Reconnect to Arbiter to list or create EVM wallets.',
        actionLabel: 'Refresh',
        onAction: () => refreshWallets(),
      ),
      _ => WalletTable(wallets: loadedWallets),
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
                isBusy: evm.isLoading,
                actions: [
                  const CreateWalletButton(),
                  SizedBox(width: 1.w),
                  const RefreshWalletButton(),
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

String _formatError(Object error) {
  final message = error.toString();
  if (message.startsWith('Exception: ')) {
    return message.substring('Exception: '.length);
  }
  return message;
}
