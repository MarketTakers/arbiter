import 'package:arbiter/features/connection/evm.dart' as evm;
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'evm.g.dart';

@riverpod
class Evm extends _$Evm {
  @override
  Future<List<WalletEntry>?> build() async {
    final connection = await ref.watch(connectionManagerProvider.future);
    if (connection == null) {
      return null;
    }

    return evm.listEvmWallets(connection);
  }

  Future<void> refreshWallets() async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      state = const AsyncData(null);
      return;
    }

    state = const AsyncLoading();
    state = await AsyncValue.guard(() => evm.listEvmWallets(connection));
  }
}

final createEvmWallet = Mutation();

Future<void> executeCreateEvmWallet(MutationTarget target) async {
  return await createEvmWallet.run(target, (tsx) async {
    final connection = await tsx.get(connectionManagerProvider.future);
    if (connection == null) {
      throw Exception('Not connected to the server.');
    }

    await evm.createEvmWallet(connection);

    await tsx.get(evmProvider.notifier).refreshWallets();
  });
}
