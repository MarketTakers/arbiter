import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:flutter/material.dart';

class WalletAccessTile extends StatelessWidget {
  const WalletAccessTile({
    super.key,
    required this.option,
    required this.value,
    required this.enabled,
    required this.onChanged,
  });

  final ClientWalletOption option;
  final bool value;
  final bool enabled;
  final VoidCallback onChanged;

  @override
  Widget build(BuildContext context) {
    return CheckboxListTile(
      contentPadding: EdgeInsets.zero,
      value: value,
      onChanged: enabled ? (_) => onChanged() : null,
      title: Text('Wallet ${option.walletId}'),
      subtitle: Text(option.address),
    );
  }
}
