import 'dart:convert';

import 'package:arbiter/features/callouts/callout_event.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'sdk_connect_approve.g.dart';

@riverpod
Stream<CalloutEvent> connectApproveEvents(Ref ref) async* {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) return;

  await for (final message in connection.outOfBandMessages) {
    switch (message.whichPayload()) {
      case UserAgentResponse_Payload.sdkClientConnectionRequest:
        final body = message.sdkClientConnectionRequest;
        final id = base64Encode(body.pubkey);
        yield CalloutEvent.added(
          id: 'connect_approve:$id',
          data: CalloutData.connectApproval(
            pubkey: id,
            clientInfo: body.info,
          ),
        );

      case UserAgentResponse_Payload.sdkClientConnectionCancel:
        final id = base64Encode(message.sdkClientConnectionCancel.pubkey);
        yield CalloutEvent.cancelled(id: 'connect_approve:$id');

      default:
        break;
    }
  }
}

Future<void> sendDecision(Ref ref, String pubkey, bool approved) async {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) return;

  final bytes = base64Decode(pubkey);

  final req = UserAgentRequest(sdkClientConnectionResponse: SdkClientConnectionResponse(
    approved: approved,
    pubkey: bytes
  ));

  await connection.tell(req);

}