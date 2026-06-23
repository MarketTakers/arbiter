import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
import 'package:arbiter/providers/sdk_clients/list.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'details.g.dart';

@riverpod
Future<ua_sdk.Entry?> clientDetails(Ref ref, int clientId) async {
  final clients = await ref.watch(sdkClientsProvider.future);
  if (clients == null) {
    return null;
  }
  for (final client in clients) {
    if (client.id == clientId) {
      return client;
    }
  }
  return null;
}
