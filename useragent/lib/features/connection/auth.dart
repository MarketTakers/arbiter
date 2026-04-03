import 'dart:async';
import 'dart:convert';

import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/features/connection/server_info_storage.dart';
import 'package:arbiter/features/identity/pk_manager.dart';
import 'package:arbiter/proto/arbiter.pbgrpc.dart';
import 'package:arbiter/proto/user_agent/auth.pb.dart' as ua_auth;
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:grpc/grpc.dart';
import 'package:mtcore/markettakers.dart';

class AuthorizationException implements Exception {
  const AuthorizationException(this.result);

  final ua_auth.AuthResult result;

  String get message => switch (result) {
    ua_auth.AuthResult.AUTH_RESULT_INVALID_KEY =>
      'Authentication failed: this device key is not registered on the server.',
    ua_auth.AuthResult.AUTH_RESULT_INVALID_SIGNATURE =>
      'Authentication failed: the server rejected the signature for this device key.',
    ua_auth.AuthResult.AUTH_RESULT_BOOTSTRAP_REQUIRED =>
      'Authentication failed: the server requires bootstrap before this device can connect.',
    ua_auth.AuthResult.AUTH_RESULT_TOKEN_INVALID =>
      'Authentication failed: the bootstrap token is invalid.',
    ua_auth.AuthResult.AUTH_RESULT_INTERNAL =>
      'Authentication failed: the server hit an internal error.',
    ua_auth.AuthResult.AUTH_RESULT_UNSPECIFIED =>
      'Authentication failed: the server returned an unspecified auth error.',
    ua_auth.AuthResult.AUTH_RESULT_SUCCESS => 'Authentication succeeded.',
    _ => 'Authentication failed: ${result.name}.',
  };

  @override
  String toString() => message;
}

class ConnectionException implements Exception {
  const ConnectionException(this.message);

  final String message;

  @override
  String toString() => message;
}

Future<Connection> connectAndAuthorize(
  StoredServerInfo serverInfo,
  KeyHandle key, {
  String? bootstrapToken,
}) async {
  Connection? connection;
  try {
    connection = await _connect(serverInfo);
    talker.info(
      'Connected to server at ${serverInfo.address}:${serverInfo.port}',
    );
    final pubkey = await key.getPublicKey();

    final req = ua_auth.AuthChallengeRequest(
      pubkey: pubkey,
      bootstrapToken: bootstrapToken,
      keyType: switch (key.alg) {
        KeyAlgorithm.rsa => ua_auth.KeyType.KEY_TYPE_RSA,
        KeyAlgorithm.ecdsa => ua_auth.KeyType.KEY_TYPE_ECDSA_SECP256K1,
        KeyAlgorithm.ed25519 => ua_auth.KeyType.KEY_TYPE_ED25519,
      },
    );
    final response = await connection.ask(
      UserAgentRequest(auth: ua_auth.Request(challengeRequest: req)),
    );
    talker.info(
      "Sent auth challenge request with pubkey ${base64Encode(pubkey)}",
    );
    talker.info('Received response from server, checking auth flow...');

    if (!response.hasAuth()) {
      throw ConnectionException(
        'Expected auth response, got ${response.whichPayload()}',
      );
    }

    final authResponse = response.auth;

    if (authResponse.hasResult()) {
      if (authResponse.result != ua_auth.AuthResult.AUTH_RESULT_SUCCESS) {
        throw AuthorizationException(authResponse.result);
      }
      talker.info('Authentication successful, connection established');
      return connection;
    }

    if (!authResponse.hasChallenge()) {
      throw ConnectionException(
        'Expected auth challenge response, got ${authResponse.whichPayload()}',
      );
    }

    final challenge = _formatChallenge(authResponse.challenge, pubkey);
    talker.info(
      'Received auth challenge, signing with key ${base64Encode(pubkey)}',
    );

    final signature = await key.sign(challenge);
    final solutionResponse = await connection.ask(
      UserAgentRequest(
        auth: ua_auth.Request(
          challengeSolution: ua_auth.AuthChallengeSolution(signature: signature),
        ),
      ),
    );

    talker.info('Sent auth challenge solution, waiting for server response...');

    if (!solutionResponse.hasAuth()) {
      throw ConnectionException(
        'Expected auth solution response, got ${solutionResponse.whichPayload()}',
      );
    }

    final authSolutionResponse = solutionResponse.auth;

    if (!authSolutionResponse.hasResult()) {
      throw ConnectionException(
        'Expected auth solution result, got ${authSolutionResponse.whichPayload()}',
      );
    }
    if (authSolutionResponse.result != ua_auth.AuthResult.AUTH_RESULT_SUCCESS) {
      throw AuthorizationException(authSolutionResponse.result);
    }

    talker.info('Authentication successful, connection established');
    return connection;
  } on AuthorizationException {
    await connection?.close();
    rethrow;
  } on GrpcError catch (error) {
    await connection?.close();
    throw ConnectionException('Failed to connect to server: ${error.message}');
  } catch (e) {
    await connection?.close();
    if (e is ConnectionException) {
      rethrow;
    }
    throw ConnectionException('Failed to connect to server: $e');
  }
}

Future<Connection> _connect(StoredServerInfo serverInfo) async {
  final channel = ClientChannel(
    serverInfo.address,
    port: serverInfo.port,
    options: ChannelOptions(
      connectTimeout: const Duration(seconds: 10),
      credentials: ChannelCredentials.secure(
        onBadCertificate: (cert, host) {
          return true;
        },
      ),
    ),
  );

  final client = ArbiterServiceClient(channel);
  final tx = StreamController<UserAgentRequest>();
  final rx = client.userAgent(tx.stream);

  return Connection(channel: channel, tx: tx, rx: rx);
}

List<int> _formatChallenge(ua_auth.AuthChallenge challenge, List<int> pubkey) {
  final encodedPubkey = base64Encode(pubkey);
  final payload = "${challenge.nonce}:$encodedPubkey";
  return utf8.encode(payload);
}
