import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

class CreateWalletButton extends ConsumerWidget {
  const CreateWalletButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final createWallet = ref.watch(createEvmWallet);
    final isCreating = createWallet is MutationPending;

    Future<void> handleCreateWallet() async {
      try {
        await executeCreateEvmWallet(ref);
        if (!context.mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('New wallet created successfully.'),
            behavior: SnackBarBehavior.floating,
          ),
        );
      } catch (e) {
        if (!context.mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to create wallet: ${_formatError(e)}'),
            behavior: SnackBarBehavior.floating,
          ),
        );
      }
    }

    return FilledButton.icon(
      onPressed: isCreating ? null : () => handleCreateWallet(),
      style: FilledButton.styleFrom(
        backgroundColor: Palette.ink,
        foregroundColor: Colors.white,
        padding: EdgeInsets.symmetric(horizontal: 1.4.w, vertical: 1.2.h),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      ),
      icon: isCreating
          ? SizedBox(
              width: 1.6.h,
              height: 1.6.h,
              child: CircularProgressIndicator(strokeWidth: 2.2),
            )
          : const Icon(Icons.add_circle_outline, size: 18),
      label: Text(isCreating ? 'Creating...' : 'Create'),
    );
  }
}

class RefreshWalletButton extends ConsumerWidget {
  const RefreshWalletButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    Future<void> handleRefreshWallets() async {
      try {
        await ref.read(evmProvider.notifier).refreshWallets();
      } catch (e) {
        if (!context.mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to refresh wallets: ${_formatError(e)}'),
            behavior: SnackBarBehavior.floating,
          ),
        );
      }
    }

    return OutlinedButton.icon(
      onPressed: () => handleRefreshWallets(),
      style: OutlinedButton.styleFrom(
        foregroundColor: Palette.ink,
        side: BorderSide(color: Palette.line),
        padding: EdgeInsets.symmetric(horizontal: 1.4.w, vertical: 1.2.h),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      ),
      icon: const Icon(Icons.refresh, size: 18),
      label: const Text('Refresh'),
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
