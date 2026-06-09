import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/client_details_state_panel.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/wallet_access_list.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/wallet_access_search_field.dart';
import 'package:arbiter/widgets/cream_frame.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class WalletAccessSection extends ConsumerWidget {
  const WalletAccessSection({
    super.key,
    required this.clientId,
    required this.state,
    required this.accessSelectionAsync,
    required this.isSavePending,
    required this.onSearchChanged,
    required this.onToggleWallet,
  });

  final int clientId;
  final ClientWalletAccessState state;
  final AsyncValue<Set<int>> accessSelectionAsync;
  final bool isSavePending;
  final ValueChanged<String> onSearchChanged;
  final ValueChanged<int> onToggleWallet;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final optionsAsync = ref.watch(clientWalletOptionsProvider);
    return CreamFrame(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Wallet access', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Text('Choose which managed wallets this client can see.'),
          const SizedBox(height: 16),
          _WalletAccessBody(
            clientId: clientId,
            state: state,
            accessSelectionAsync: accessSelectionAsync,
            isSavePending: isSavePending,
            optionsAsync: optionsAsync,
            onSearchChanged: onSearchChanged,
            onToggleWallet: onToggleWallet,
          ),
        ],
      ),
    );
  }
}

class _WalletAccessBody extends StatelessWidget {
  const _WalletAccessBody({
    required this.clientId,
    required this.state,
    required this.accessSelectionAsync,
    required this.isSavePending,
    required this.optionsAsync,
    required this.onSearchChanged,
    required this.onToggleWallet,
  });

  final int clientId;
  final ClientWalletAccessState state;
  final AsyncValue<Set<int>> accessSelectionAsync;
  final bool isSavePending;
  final AsyncValue<List<ClientWalletOption>> optionsAsync;
  final ValueChanged<String> onSearchChanged;
  final ValueChanged<int> onToggleWallet;

  @override
  Widget build(BuildContext context) {
    final selectionState = accessSelectionAsync;
    if (selectionState.isLoading) {
      return const ClientDetailsStatePanel(
        title: 'Loading wallet access',
        body: 'Pulling the current wallet permissions for this client.',
        icon: Icons.hourglass_top,
      );
    }
    if (selectionState.hasError) {
      return ClientDetailsStatePanel(
        title: 'Wallet access unavailable',
        body: selectionState.error.toString(),
        icon: Icons.lock_outline,
      );
    }
    return optionsAsync.when(
      data: (options) => _WalletAccessLoaded(
        state: state,
        isSavePending: isSavePending,
        options: options,
        onSearchChanged: onSearchChanged,
        onToggleWallet: onToggleWallet,
      ),
      error: (error, _) => ClientDetailsStatePanel(
        title: 'Wallet list unavailable',
        body: error.toString(),
        icon: Icons.sync_problem,
      ),
      loading: () => const ClientDetailsStatePanel(
        title: 'Loading wallets',
        body: 'Pulling the managed wallet inventory.',
        icon: Icons.hourglass_top,
      ),
    );
  }
}

class _WalletAccessLoaded extends StatelessWidget {
  const _WalletAccessLoaded({
    required this.state,
    required this.isSavePending,
    required this.options,
    required this.onSearchChanged,
    required this.onToggleWallet,
  });

  final ClientWalletAccessState state;
  final bool isSavePending;
  final List<ClientWalletOption> options;
  final ValueChanged<String> onSearchChanged;
  final ValueChanged<int> onToggleWallet;

  @override
  Widget build(BuildContext context) {
    if (options.isEmpty) {
      return const ClientDetailsStatePanel(
        title: 'No wallets yet',
        body: 'Create a managed wallet before assigning client access.',
        icon: Icons.account_balance_wallet_outlined,
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        WalletAccessSearchField(
          searchQuery: state.searchQuery,
          onChanged: onSearchChanged,
        ),
        const SizedBox(height: 16),
        WalletAccessList(
          options: _filterOptions(options, state.searchQuery),
          selectedWalletIds: state.selectedWalletIds,
          enabled: !isSavePending,
          onToggleWallet: onToggleWallet,
        ),
      ],
    );
  }
}

List<ClientWalletOption> _filterOptions(
  List<ClientWalletOption> options,
  String query,
) {
  if (query.isEmpty) {
    return options;
  }
  final normalized = query.toLowerCase();
  return options
      .where((option) => option.address.toLowerCase().contains(normalized))
      .toList(growable: false);
}
