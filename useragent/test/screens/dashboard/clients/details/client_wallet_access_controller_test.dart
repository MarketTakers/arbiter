import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class _SuccessRepository implements ClientWalletAccessRepository {
  Set<int>? savedWalletIds;

  @override
  Future<Set<int>> fetchSelectedWalletIds(int clientId) async => {1};

  @override
  Future<void> saveSelectedWalletIds(int clientId, Set<int> walletIds) async {
    savedWalletIds = walletIds;
  }
}

class _FailureRepository implements ClientWalletAccessRepository {
  @override
  Future<Set<int>> fetchSelectedWalletIds(int clientId) async => const {};

  @override
  Future<void> saveSelectedWalletIds(int clientId, Set<int> walletIds) async {
    throw UnsupportedError('Not supported yet: $walletIds');
  }
}

void main() {
  test('save updates the original selection after toggles', () async {
    final repository = _SuccessRepository();
    final container = ProviderContainer(
      overrides: [
        clientWalletAccessRepositoryProvider.overrideWithValue(repository),
      ],
    );
    addTearDown(container.dispose);

    final controller = container.read(
      clientWalletAccessControllerProvider(42).notifier,
    );
    await container.read(clientWalletAccessSelectionProvider(42).future);
    controller.toggleWallet(2);

    expect(
      container
          .read(clientWalletAccessControllerProvider(42))
          .selectedWalletIds,
      {1, 2},
    );
    expect(
      container.read(clientWalletAccessControllerProvider(42)).hasChanges,
      isTrue,
    );

    await executeSaveClientWalletAccess(container, clientId: 42);

    expect(repository.savedWalletIds, {1, 2});
    expect(
      container
          .read(clientWalletAccessControllerProvider(42))
          .originalWalletIds,
      {1, 2},
    );
    expect(
      container.read(clientWalletAccessControllerProvider(42)).hasChanges,
      isFalse,
    );
  });

  test('save failure preserves edits and exposes a mutation error', () async {
    final container = ProviderContainer(
      overrides: [
        clientWalletAccessRepositoryProvider.overrideWithValue(
          _FailureRepository(),
        ),
      ],
    );
    addTearDown(container.dispose);

    final controller = container.read(
      clientWalletAccessControllerProvider(42).notifier,
    );
    await container.read(clientWalletAccessSelectionProvider(42).future);
    controller.toggleWallet(3);
    await expectLater(
      executeSaveClientWalletAccess(container, clientId: 42),
      throwsUnsupportedError,
    );

    expect(
      container
          .read(clientWalletAccessControllerProvider(42))
          .selectedWalletIds,
      {3},
    );
    expect(
      container.read(clientWalletAccessControllerProvider(42)).hasChanges,
      isTrue,
    );
    expect(
      container.read(saveClientWalletAccessMutation(42)),
      isA<MutationError<void>>(),
    );
  });
}
