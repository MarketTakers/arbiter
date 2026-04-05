import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
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

    final accessById = <int, ua_sdk.WalletAccessEntry>{
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
            Container(
              width: 0.8.w,
              decoration: BoxDecoration(
                color: accent,
                borderRadius: const BorderRadius.horizontal(
                  left: Radius.circular(24),
                ),
              ),
            ),
            Expanded(
              child: Padding(
                padding: EdgeInsets.symmetric(
                  horizontal: 1.6.w,
                  vertical: 1.4.h,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
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
