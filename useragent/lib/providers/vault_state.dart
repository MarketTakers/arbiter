import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:mtcore/markettakers.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'vault_state.g.dart';

@riverpod
Future<VaultState?> vaultState(Ref ref) async {
  final conn = await ref.watch(connectionManagerProvider.future);
  if (conn == null) {
    return null;
  }

  final resp = await conn.request(UserAgentRequest(queryVaultState: Empty()));
  if (resp.whichPayload() != UserAgentResponse_Payload.vaultState) {
    talker.warning('Expected vault state response, got ${resp.whichPayload()}');
    return null;
  }

  final vaultState = resp.vaultState;

  return vaultState;
}
