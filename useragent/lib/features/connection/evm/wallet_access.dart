import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';

Future<Set<int>> readClientWalletAccess(
  Connection connection, {
  required int clientId,
}) async {
  final response = await connection.ask(
    UserAgentRequest(sdkClient: ua_sdk.Request(listWalletAccess: Empty())),
  );
  if (!response.hasSdkClient()) {
    throw Exception(
      'Expected SDK client response, got ${response.whichPayload()}',
    );
  }
  final sdkClientResponse = response.sdkClient;
  if (!sdkClientResponse.hasListWalletAccess()) {
    throw Exception(
      'Expected list wallet access response, got ${sdkClientResponse.whichPayload()}',
    );
  }
  return {
    for (final entry in sdkClientResponse.listWalletAccess.accesses)
      if (entry.access.sdkClientId == clientId) entry.access.walletId,
  };
}

Future<List<ua_sdk.WalletAccessEntry>> listAllWalletAccesses(
  Connection connection,
) async {
  final response = await connection.ask(
    UserAgentRequest(sdkClient: ua_sdk.Request(listWalletAccess: Empty())),
  );
  if (!response.hasSdkClient()) {
    throw Exception(
      'Expected SDK client response, got ${response.whichPayload()}',
    );
  }
  final sdkClientResponse = response.sdkClient;
  if (!sdkClientResponse.hasListWalletAccess()) {
    throw Exception(
      'Expected list wallet access response, got ${sdkClientResponse.whichPayload()}',
    );
  }
  return sdkClientResponse.listWalletAccess.accesses.toList(growable: false);
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
        sdkClient: ua_sdk.Request(
          grantWalletAccess: ua_sdk.GrantWalletAccess(
            accesses: [
              for (final walletId in toGrant)
                ua_sdk.WalletAccess(sdkClientId: clientId, walletId: walletId),
            ],
          ),
        ),
      ),
    );
  }

  if (toRevoke.isNotEmpty) {
    await connection.tell(
      UserAgentRequest(
        sdkClient: ua_sdk.Request(
          revokeWalletAccess: ua_sdk.RevokeWalletAccess(
            accesses: [for (final walletId in toRevoke) walletId],
          ),
        ),
      ),
    );
  }
}
