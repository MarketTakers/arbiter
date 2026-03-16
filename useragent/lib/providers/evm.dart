import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
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

    return listEvmWallets(connection);
  }

  Future<void> refreshWallets() async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      state = const AsyncData(null);
      return;
    }

    state = const AsyncLoading();
    state = await AsyncValue.guard(() => listEvmWallets(connection));
  }

  Future<void> createWallet() async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      throw Exception('Not connected to the server.');
    }

    await createEvmWallet(connection);
    state = await AsyncValue.guard(() => listEvmWallets(connection));
  }
}
