import 'dart:async';

import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:grpc/grpc.dart';
import 'package:mtcore/markettakers.dart';

class Connection {
  final ClientChannel channel;
  final StreamController<UserAgentRequest> _tx;
  final StreamIterator<UserAgentResponse> _rx;

  Connection({
    required this.channel,
    required StreamController<UserAgentRequest> tx,
    required ResponseStream<UserAgentResponse> rx,
  }) : _tx = tx,
       _rx = StreamIterator(rx);

  Future<void> send(UserAgentRequest request) async {
    talker.debug('Sending request: ${request.toDebugString()}');
    _tx.add(request);
  }

  Future<UserAgentResponse> receive() async {
    final hasValue = await _rx.moveNext();
    if (!hasValue) {
      throw Exception('Connection closed while waiting for server response.');
    }
    talker.debug('Received response: ${_rx.current.toDebugString()}');
    return _rx.current;
  }

  Future<void> close() async {
    await _tx.close();
    await channel.shutdown();
  }
}
