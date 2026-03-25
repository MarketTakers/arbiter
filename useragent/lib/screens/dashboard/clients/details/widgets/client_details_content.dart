import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/client_details_header.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/client_summary_card.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/wallet_access_save_bar.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/wallet_access_section.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class ClientDetailsContent extends ConsumerWidget {
  const ClientDetailsContent({
    super.key,
    required this.clientId,
    required this.client,
  });

  final int clientId;
  final SdkClientEntry client;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(clientWalletAccessControllerProvider(clientId));
    final notifier = ref.read(
      clientWalletAccessControllerProvider(clientId).notifier,
    );
    final saveMutation = ref.watch(saveClientWalletAccessMutation(clientId));
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const ClientDetailsHeader(),
        const SizedBox(height: 16),
        ClientSummaryCard(client: client),
        const SizedBox(height: 16),
        WalletAccessSection(
          clientId: clientId,
          state: state,
          accessSelectionAsync: ref.watch(
            clientWalletAccessSelectionProvider(clientId),
          ),
          isSavePending: saveMutation is MutationPending,
          onSearchChanged: notifier.setSearchQuery,
          onToggleWallet: notifier.toggleWallet,
        ),
        const SizedBox(height: 16),
        WalletAccessSaveBar(
          state: state,
          saveMutation: saveMutation,
          onDiscard: notifier.discardChanges,
          onSave: () => executeSaveClientWalletAccess(ref, clientId: clientId),
        ),
      ],
    );
  }
}
