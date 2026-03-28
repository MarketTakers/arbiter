import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';

Future<Set<int>> readClientWalletAccess(
  Connection connection, {
  required int clientId,
}) async {
  final response = await connection.ask(
    UserAgentRequest(listWalletAccess: Empty()),
  );
  if (!response.hasListWalletAccessResponse()) {
    throw Exception(
      'Expected list wallet access response, got ${response.whichPayload()}',
    );
  }
  return {
    for (final entry in response.listWalletAccessResponse.accesses)
      if (entry.access != null && entry.access.sdkClientId == clientId) entry.access.walletId,
  };
}

Future<void> writeClientWalletAccess(
  Connection connection, {
  required int clientId,
  required Set<int> walletIds,
}) async {
  final current = await readClientWalletAccess(connection, clientId: clientId);

  final toGrant = walletIds.difference(current);
  final toRevoke = current.difference(walletIds);

  if (toGrant.isNotEmpty) {
    await connection.tell(
      UserAgentRequest(
        grantWalletAccess: SdkClientGrantWalletAccess(
          accesses: [
            for (final walletId in toGrant)
              WalletAccess(sdkClientId: clientId, walletId: walletId),
          ],
        ),
      ),
    );
  }

  if (toRevoke.isNotEmpty) {
    await connection.tell(
      UserAgentRequest(
        revokeWalletAccess: SdkClientRevokeWalletAccess(
          accesses: [
            for (final walletId in toRevoke)
              walletId
          ],
        ),
      ),
    );
  }
}
