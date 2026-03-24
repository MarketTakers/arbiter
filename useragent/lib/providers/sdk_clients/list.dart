import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'list.g.dart';

@riverpod
Future<List<SdkClientEntry>?> sdkClients(Ref ref) async {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) {
    return null;
  }

  final resp = await connection.ask(
    UserAgentRequest(sdkClientList: Empty()),
  );

  if (!resp.hasSdkClientListResponse()) {
    throw Exception(
      'Expected SDK client list response, got ${resp.whichPayload()}',
    );
  }
  final result = resp.sdkClientListResponse;

  switch (result.whichResult()) {
    case SdkClientListResponse_Result.clients:
      return result.clients.clients.toList(growable: false);
    case SdkClientListResponse_Result.error:
      throw Exception('Error listing SDK clients: ${result.error}');
    case SdkClientListResponse_Result.notSet:
      throw Exception('SDK client list response was empty.');
  }
}
