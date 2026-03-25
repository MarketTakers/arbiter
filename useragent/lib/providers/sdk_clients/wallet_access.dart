
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:mtcore/markettakers.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'wallet_access.g.dart';

@riverpod
Future<List<SdkClientWalletAccess>?> walletAccess(Ref ref) async {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) {
    return null;
  }

  final accesses = await connection.ask(UserAgentRequest(listWalletAccess: Empty()));

  if (accesses.hasListWalletAccessResponse()) {
    return accesses.listWalletAccessResponse.accesses.toList();
  } else {
    talker.warning('Received unexpected response for listWalletAccess: $accesses');
    return null;
  }
}
