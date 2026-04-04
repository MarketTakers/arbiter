import 'package:arbiter/proto/shared/client.pb.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'callout_event.freezed.dart';

@freezed
sealed class CalloutData with _$CalloutData {
  const factory CalloutData.connectApproval({
    required String pubkey,
    required ClientInfo clientInfo,
  }) = ConnectApprovalData;
}

@freezed
sealed class CalloutEvent with _$CalloutEvent {
  const factory CalloutEvent.added({
    required String id,
    required CalloutData data,
  }) = CalloutEventAdded;

  const factory CalloutEvent.cancelled({
    required String id,
  }) = CalloutEventCancelled;
}
