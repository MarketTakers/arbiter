import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/evm/evm_grants.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access_list.dart';
import 'package:arbiter/router.gr.dart';
import 'package:arbiter/screens/dashboard/evm/grants/widgets/grant_card.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:arbiter/widgets/page_header.dart';
import 'package:arbiter/widgets/state_panel.dart';
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

class _GrantList extends StatelessWidget {
  const _GrantList({required this.grants});

  final List<GrantEntry> grants;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: Column(
        children: [
          for (var i = 0; i < grants.length; i++)
            Padding(
              padding: EdgeInsets.only(
                bottom: i == grants.length - 1 ? 0 : 1.8.h,
              ),
              child: GrantCard(grant: grants[i]),
            ),
        ],
      ),
    );
  }
}

@RoutePage()
class EvmGrantsScreen extends ConsumerWidget {
  const EvmGrantsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Screen watches only the grant list for top-level state decisions
    final grantsAsync = ref.watch(evmGrantsProvider);

    Future<void> refresh() async {
      ref.invalidate(walletAccessListProvider);
      ref.invalidate(evmGrantsProvider);
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
      AsyncLoading() when grantsState == null => const StatePanel(
        icon: Icons.hourglass_top,
        title: 'Loading grants',
        body: 'Pulling grant registry from Arbiter.',
        busy: true,
      ),
      AsyncError(:final error) => StatePanel(
        icon: Icons.sync_problem,
        title: 'Grant registry unavailable',
        body: _formatError(error),
        actionLabel: 'Retry',
        onAction: safeRefresh,
      ),
      AsyncData(:final value) when value == null => StatePanel(
        icon: Icons.portable_wifi_off,
        title: 'No active server connection',
        body: 'Reconnect to Arbiter to list EVM grants.',
        actionLabel: 'Refresh',
        onAction: safeRefresh,
      ),
      _ when grants != null && grants.isEmpty => StatePanel(
        icon: Icons.policy_outlined,
        title: 'No grants yet',
        body: 'Create a grant to allow SDK clients to sign transactions.',
        actionLabel: 'Create grant',
        onAction: () async => context.router.push(const CreateEvmGrantRoute()),
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
