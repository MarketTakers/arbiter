import 'dart:async';
import 'dart:convert';

import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/features/connection/server_info_storage.dart';
import 'package:arbiter/features/identity/pk_manager.dart';
import 'package:arbiter/proto/arbiter.pbgrpc.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:grpc/grpc.dart';
import 'package:mtcore/markettakers.dart';

Future<Connection> connectAndAuthorize(
  StoredServerInfo serverInfo,
  KeyHandle key, {
  String? bootstrapToken,
}) async {
  try {
    final connection = await _connect(serverInfo);
    talker.info(
      'Connected to server at ${serverInfo.address}:${serverInfo.port}',
    );
    final pubkey = await key.getPublicKey();

    final req = AuthChallengeRequest(
      pubkey: pubkey,
      bootstrapToken: bootstrapToken,
      keyType: switch (key.alg) {
        KeyAlgorithm.rsa => KeyType.KEY_TYPE_RSA,
        KeyAlgorithm.ecdsa => KeyType.KEY_TYPE_ECDSA_SECP256K1,
        KeyAlgorithm.ed25519 => KeyType.KEY_TYPE_ED25519,
      },
    );
    final response = await connection.request(
      UserAgentRequest(authChallengeRequest: req),
    );
    talker.info(
      "Sent auth challenge request with pubkey ${base64Encode(pubkey)}",
    );
    talker.info('Received response from server, checking auth flow...');

    if (response.hasAuthResult()) {
      if (response.authResult != AuthResult.AUTH_RESULT_SUCCESS) {
        throw Exception('Authentication failed: ${response.authResult}');
      }
      talker.info('Authentication successful, connection established');
      return connection;
    }

    if (!response.hasAuthChallenge()) {
      throw Exception(
        'Expected AuthChallengeResponse, got ${response.whichPayload()}',
      );
    }

    final challenge = _formatChallenge(response.authChallenge, pubkey);
    talker.info(
      'Received auth challenge, signing with key ${base64Encode(pubkey)}',
    );

    final signature = await key.sign(challenge);
    final solutionResponse = await connection.request(
      UserAgentRequest(authChallengeSolution: AuthChallengeSolution(signature: signature)),
    );

    talker.info('Sent auth challenge solution, waiting for server response...');

    if (!solutionResponse.hasAuthResult()) {
      throw Exception(
        'Expected AuthChallengeSolutionResponse, got ${solutionResponse.whichPayload()}',
      );
    }
    if (solutionResponse.authResult != AuthResult.AUTH_RESULT_SUCCESS) {
      throw Exception('Authentication failed: ${solutionResponse.authResult}');
    }

    talker.info('Authentication successful, connection established');
    return connection;
  } catch (e) {
    throw Exception('Failed to connect to server: $e');
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

List<int> _formatChallenge(AuthChallenge challenge, List<int> pubkey) {
  final encodedPubkey = base64Encode(pubkey);
  final payload = "${challenge.nonce}:$encodedPubkey";
  return utf8.encode(payload);
}
