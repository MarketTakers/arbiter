import 'package:arbiter/features/connection/evm/wallet_access.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:arbiter/providers/evm/evm.dart';
import 'package:flutter/foundation.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'wallet_access.g.dart';

class ClientWalletOption {
  const ClientWalletOption({required this.walletId, required this.address});

  final int walletId;
  final String address;
}

class ClientWalletAccessState {
  const ClientWalletAccessState({
    this.searchQuery = '',
    this.originalWalletIds = const {},
    this.selectedWalletIds = const {},
  });

  final String searchQuery;
  final Set<int> originalWalletIds;
  final Set<int> selectedWalletIds;

  bool get hasChanges => !setEquals(originalWalletIds, selectedWalletIds);

  ClientWalletAccessState copyWith({
    String? searchQuery,
    Set<int>? originalWalletIds,
    Set<int>? selectedWalletIds,
  }) {
    return ClientWalletAccessState(
      searchQuery: searchQuery ?? this.searchQuery,
      originalWalletIds: originalWalletIds ?? this.originalWalletIds,
      selectedWalletIds: selectedWalletIds ?? this.selectedWalletIds,
    );
  }
}

final saveClientWalletAccessMutation = Mutation<void>();

abstract class ClientWalletAccessRepository {
  Future<Set<int>> fetchSelectedWalletIds(int clientId);
  Future<void> saveSelectedWalletIds(int clientId, Set<int> walletIds);
}

class ServerClientWalletAccessRepository
    implements ClientWalletAccessRepository {
  ServerClientWalletAccessRepository(this.ref);

  final Ref ref;

  @override
  Future<Set<int>> fetchSelectedWalletIds(int clientId) async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      throw Exception('Not connected to the server.');
    }
    return readClientWalletAccess(connection, clientId: clientId);
  }

  @override
  Future<void> saveSelectedWalletIds(int clientId, Set<int> walletIds) async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      throw Exception('Not connected to the server.');
    }
    await writeClientWalletAccess(
      connection,
      clientId: clientId,
      walletIds: walletIds,
    );
  }
}

@riverpod
ClientWalletAccessRepository clientWalletAccessRepository(Ref ref) {
  return ServerClientWalletAccessRepository(ref);
}

@riverpod
Future<List<ClientWalletOption>> clientWalletOptions(Ref ref) async {
  final wallets = await ref.watch(evmProvider.future) ?? const <WalletEntry>[];
  return [
    for (var index = 0; index < wallets.length; index++)
      ClientWalletOption(
        walletId: index + 1,
        address: formatWalletAddress(wallets[index].address),
      ),
  ];
}

@riverpod
Future<Set<int>> clientWalletAccessSelection(Ref ref, int clientId) async {
  final repository = ref.watch(clientWalletAccessRepositoryProvider);
  return repository.fetchSelectedWalletIds(clientId);
}

@riverpod
class ClientWalletAccessController extends _$ClientWalletAccessController {
  @override
  ClientWalletAccessState build(int clientId) {
    final selection = ref.read(clientWalletAccessSelectionProvider(clientId));

    void sync(AsyncValue<Set<int>> value) {
      value.when(data: hydrate, error: (_, _) {}, loading: () {});
    }

    ref.listen<AsyncValue<Set<int>>>(
      clientWalletAccessSelectionProvider(clientId),
      (_, next) => sync(next),
    );
    return selection.when(
      data: (walletIds) => ClientWalletAccessState(
        originalWalletIds: Set.of(walletIds),
        selectedWalletIds: Set.of(walletIds),
      ),
      error: (error, _) => const ClientWalletAccessState(),
      loading: () => const ClientWalletAccessState(),
    );
  }

  void hydrate(Set<int> selectedWalletIds) {
    state = state.copyWith(
      originalWalletIds: Set.of(selectedWalletIds),
      selectedWalletIds: Set.of(selectedWalletIds),
    );
  }

  void setSearchQuery(String value) {
    state = state.copyWith(searchQuery: value);
  }

  void toggleWallet(int walletId) {
    final next = Set<int>.of(state.selectedWalletIds);
    if (!next.add(walletId)) {
      next.remove(walletId);
    }
    state = state.copyWith(selectedWalletIds: next);
  }

  void discardChanges() {
    state = state.copyWith(selectedWalletIds: Set.of(state.originalWalletIds));
  }
}

Future<void> executeSaveClientWalletAccess(
  MutationTarget ref, {
  required int clientId,
}) {
  final mutation = saveClientWalletAccessMutation(clientId);
  return mutation.run(ref, (tsx) async {
    final repository = tsx.get(clientWalletAccessRepositoryProvider);
    final controller = tsx.get(
      clientWalletAccessControllerProvider(clientId).notifier,
    );
    final selectedWalletIds = tsx
        .get(clientWalletAccessControllerProvider(clientId))
        .selectedWalletIds;
    await repository.saveSelectedWalletIds(clientId, selectedWalletIds);
    controller.hydrate(selectedWalletIds);
  });
}

String formatWalletAddress(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '0x$hex';
}
