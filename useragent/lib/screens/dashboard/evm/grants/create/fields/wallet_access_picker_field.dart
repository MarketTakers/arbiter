// lib/screens/dashboard/evm/grants/create/fields/wallet_access_picker_field.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access_list.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/provider.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class WalletAccessPickerField extends ConsumerWidget {
  const WalletAccessPickerField({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(grantCreationProvider);
    final allAccesses =
        ref.watch(walletAccessListProvider).asData?.value ??
        const <ua_sdk.WalletAccessEntry>[];
    final wallets =
        ref.watch(evmProvider).asData?.value ?? const <WalletEntry>[];

    final walletById = <int, WalletEntry>{for (final w in wallets) w.id: w};
    final accesses = state.selectedClientId == null
        ? const <ua_sdk.WalletAccessEntry>[]
        : allAccesses
              .where((a) => a.access.sdkClientId == state.selectedClientId)
              .toList();

    return FormBuilderDropdown<int>(
      name: 'walletAccessId',
      enabled: accesses.isNotEmpty,
      decoration: InputDecoration(
        labelText: 'Wallet access',
        helperText: state.selectedClientId == null
            ? 'Select a client first'
            : accesses.isEmpty
            ? 'No wallet accesses for this client'
            : null,
        border: const OutlineInputBorder(),
      ),
      items: [
        for (final a in accesses)
          DropdownMenuItem(
            value: a.id,
            child: Text(() {
              final wallet = walletById[a.access.walletId];
              return wallet != null
                  ? shortAddress(wallet.address)
                  : 'Wallet #${a.access.walletId}';
            }()),
          ),
      ],
    );
  }
}
