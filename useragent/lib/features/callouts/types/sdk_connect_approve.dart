import 'dart:convert';

import 'package:arbiter/features/callouts/callout_event.dart';
import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
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
      case UserAgentResponse_Payload.sdkClient:
        final sdkClientMessage = message.sdkClient;
        switch (sdkClientMessage.whichPayload()) {
          case ua_sdk.Response_Payload.connectionRequest:
            final body = sdkClientMessage.connectionRequest;
            final id = base64Encode(body.pubkey);
            yield CalloutEvent.added(
              id: 'connect_approve:$id',
              data: CalloutData.connectApproval(
                pubkey: id,
                clientInfo: body.info,
              ),
            );

          case ua_sdk.Response_Payload.connectionCancel:
            final id = base64Encode(sdkClientMessage.connectionCancel.pubkey);
            yield CalloutEvent.cancelled(id: 'connect_approve:$id');

          default:
            break;
        }

      default:
        break;
    }
  }
}

Future<void> sendDecision(Ref ref, String pubkey, bool approved) async {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) return;

  final bytes = base64Decode(pubkey);

  final req = UserAgentRequest(
    sdkClient: ua_sdk.Request(
      connectionResponse: ua_sdk.ConnectionResponse(
        approved: approved,
        pubkey: bytes,
      ),
    ),
  );

  await connection.tell(req);
}
