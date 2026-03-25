import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/wallet_access_tile.dart';
import 'package:flutter/material.dart';

class WalletAccessList extends StatelessWidget {
  const WalletAccessList({
    super.key,
    required this.options,
    required this.selectedWalletIds,
    required this.enabled,
    required this.onToggleWallet,
  });

  final List<ClientWalletOption> options;
  final Set<int> selectedWalletIds;
  final bool enabled;
  final ValueChanged<int> onToggleWallet;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        for (final option in options)
          WalletAccessTile(
            option: option,
            value: selectedWalletIds.contains(option.walletId),
            enabled: enabled,
            onChanged: () => onToggleWallet(option.walletId),
          ),
      ],
    );
  }
}
