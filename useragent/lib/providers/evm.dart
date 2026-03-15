import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'evm.g.dart';

@riverpod
class Evm extends _$Evm {
  Future<List<WalletEntry>?> build() async {
    final connection = await ref.watch(connectionManagerProvider.future);
    if (connection == null) {
      return null;
    }

    await connection.send(UserAgentRequest(
      evmWalletList: Empty()
    ));

    final response = await connection.receive();
  }
}
