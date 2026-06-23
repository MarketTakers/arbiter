import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:arbiter/widgets/cream_frame.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';

class WalletAccessSaveBar extends StatelessWidget {
  const WalletAccessSaveBar({
    super.key,
    required this.state,
    required this.saveMutation,
    required this.onDiscard,
    required this.onSave,
  });

  final ClientWalletAccessState state;
  final MutationState<void> saveMutation;
  final VoidCallback onDiscard;
  final Future<void> Function() onSave;

  @override
  Widget build(BuildContext context) {
    final isPending = saveMutation is MutationPending;
    final errorText = switch (saveMutation) {
      MutationError(:final error) => error.toString(),
      _ => null,
    };
    return CreamFrame(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (errorText != null) ...[
            Text(errorText, style: TextStyle(color: Palette.coral)),
            const SizedBox(height: 12),
          ],
          Row(
            children: [
              TextButton(
                onPressed: state.hasChanges && !isPending ? onDiscard : null,
                child: const Text('Reset'),
              ),
              const Spacer(),
              FilledButton(
                onPressed: state.hasChanges && !isPending ? onSave : null,
                child: Text(isPending ? 'Saving...' : 'Save changes'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
