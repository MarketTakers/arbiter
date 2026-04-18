import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'list.g.dart';

@riverpod
Future<List<ua_sdk.Entry>?> sdkClients(Ref ref) async {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) {
    return null;
  }

  final resp = await connection.ask(
    UserAgentRequest(sdkClient: ua_sdk.Request(list: Empty())),
  );

  if (!resp.hasSdkClient()) {
    throw Exception('Expected SDK client response, got ${resp.whichPayload()}');
  }
  final sdkClientResponse = resp.sdkClient;
  if (!sdkClientResponse.hasList()) {
    throw Exception(
      'Expected SDK client list response, got ${sdkClientResponse.whichPayload()}',
    );
  }
  final result = sdkClientResponse.list;

  switch (result.whichResult()) {
    case ua_sdk.ListResponse_Result.clients:
      return result.clients.clients.toList(growable: false);
    case ua_sdk.ListResponse_Result.error:
      throw Exception('Error listing SDK clients: ${result.error}');
    case ua_sdk.ListResponse_Result.notSet:
      throw Exception('SDK client list response was empty.');
  }
}
