import 'package:arbiter/features/callouts/active_callout.dart';
import 'package:arbiter/features/callouts/callout_event.dart';
import 'package:arbiter/features/callouts/types/sdk_connect_approve.dart'
    as connect_approve;
import 'package:arbiter/proto/client.pb.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'callout_manager.g.dart';

@Riverpod(keepAlive: true)
class CalloutManager extends _$CalloutManager {
  @override
  Map<String, ActiveCallout> build() {
    ref.listen(connect_approve.connectApproveEventsProvider, (_, next) {
      next.whenData(_processEvent);
    });
    return {};
  }

  void _processEvent(CalloutEvent event) {
    switch (event) {
      case CalloutEventAdded(:final id, :final data):
        state = {...state, id: _toActiveCallout(id, data)};
      case CalloutEventCancelled(:final id):
        state = {...state}..remove(id);
    }
  }

  Future<void> sendDecision(String id, bool approved) async {
    final callout = state[id];
    if (callout == null) return;
    switch (callout.data) {
      case ConnectApprovalData(:final pubkey):
        await connect_approve.sendDecision(ref, pubkey, approved);
    }
    dismiss(id);
  }

  void dismiss(String id) {
    state = {...state}..remove(id);
  }
}

ActiveCallout _toActiveCallout(String id, CalloutData data) => switch (data) {
  ConnectApprovalData(:final clientInfo) => ActiveCallout(
    id: id,
    title: 'Connection Request',
    description: _clientDisplayName(clientInfo) != null
        ? '${_clientDisplayName(clientInfo)} is requesting a connection.'
        : 'An SDK client is requesting a connection.',
    addedAt: DateTime.now(),
    data: data,
  ),
};

String? _clientDisplayName(ClientInfo info) =>
    info.hasName() && info.name.isNotEmpty ? info.name : null;
