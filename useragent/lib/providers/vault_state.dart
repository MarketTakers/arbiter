import 'package:arbiter/proto/shared/vault.pbenum.dart';
import 'package:arbiter/proto/user_agent/vault/vault.pb.dart' as ua_vault;
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

  final resp = await conn.ask(
    UserAgentRequest(vault: ua_vault.Request(queryState: Empty())),
  );
  if (!resp.hasVault()) {
    talker.warning('Expected vault state response, got ${resp.whichPayload()}');
    return null;
  }

  final vaultResponse = resp.vault;
  if (!vaultResponse.hasState()) {
    talker.warning(
      'Expected vault state payload, got ${vaultResponse.whichPayload()}',
    );
    return null;
  }

  final vaultState = vaultResponse.state;

  return vaultState;
}
