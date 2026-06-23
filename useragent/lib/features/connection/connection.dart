import 'dart:async';

import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:grpc/grpc.dart';
import 'package:mtcore/markettakers.dart';

class Connection {
  Connection({
    required this.channel,
    required StreamController<UserAgentRequest> tx,
    required ResponseStream<UserAgentResponse> rx,
  }) : _tx = tx {
    _rxSubscription = rx.listen(
      _handleResponse,
      onError: _handleError,
      onDone: _handleDone,
      cancelOnError: true,
    );
  }

  final ClientChannel channel;
  final StreamController<UserAgentRequest> _tx;
  final Map<int, Completer<UserAgentResponse>> _pendingRequests = {};
  final StreamController<UserAgentResponse> _outOfBandMessages =
      StreamController<UserAgentResponse>.broadcast();

  StreamSubscription<UserAgentResponse>? _rxSubscription;
  int _nextRequestId = 0;

  Stream<UserAgentResponse> get outOfBandMessages => _outOfBandMessages.stream;

  Future<UserAgentResponse> ask(UserAgentRequest message) async {
    _ensureOpen();

    final requestId = _nextRequestId++;
    final completer = Completer<UserAgentResponse>();
    _pendingRequests[requestId] = completer;

    message.id = requestId;
    talker.debug('Sending request: ${message.toDebugString()}');

    try {
      _tx.add(message);
    } catch (error, stackTrace) {
      _pendingRequests.remove(requestId);
      completer.completeError(error, stackTrace);
    }

    return completer.future;
  }

  Future<void> tell(UserAgentRequest message) async {
    _ensureOpen();

    final requestId = _nextRequestId++;
    message.id = requestId;

    talker.debug('Sending message: ${message.toDebugString()}');

    try {
      _tx.add(message);
    } catch (error, stackTrace) {
      talker.error('Failed to send message: $error', error, stackTrace);
    }
  }

  Future<void> close() async {
    talker.debug('Closing connection...');
    final rxSubscription = _rxSubscription;
    if (rxSubscription == null) {
      return;
    }

    _rxSubscription = null;
    await rxSubscription.cancel();
    _failPendingRequests(Exception('Connection closed.'));
    await _outOfBandMessages.close();
    await _tx.close();
    await channel.shutdown();
  }

  void _handleResponse(UserAgentResponse response) {
    talker.debug('Received response: ${response.toDebugString()}');

    if (response.hasId()) {
      final completer = _pendingRequests.remove(response.id);
      if (completer == null) {
        talker.warning(
          'Received response for unknown request id ${response.id}',
        );
        return;
      }
      completer.complete(response);
      return;
    }

    _outOfBandMessages.add(response);
  }

  void _handleError(Object error, StackTrace stackTrace) {
    _rxSubscription = null;
    _failPendingRequests(error, stackTrace);
    _outOfBandMessages.addError(error, stackTrace);
  }

  void _handleDone() {
    talker.debug('Connection closed by server.');
    if (_rxSubscription == null) {
      return;
    }

    _rxSubscription = null;
    final error = Exception(
      'Connection closed while waiting for server response.',
    );
    _failPendingRequests(error);
    _outOfBandMessages.close();
  }

  void _failPendingRequests(Object error, [StackTrace? stackTrace]) {
    final pendingRequests = _pendingRequests.values.toList(growable: false);
    _pendingRequests.clear();

    for (final completer in pendingRequests) {
      if (!completer.isCompleted) {
        completer.completeError(error, stackTrace);
      }
    }
  }

  void _ensureOpen() {
    if (_rxSubscription == null) {
      throw StateError('Connection is closed.');
    }
  }
}
