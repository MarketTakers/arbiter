import 'dart:async';
import 'dart:convert';

import 'package:arbiter/features/connection/server_info_storage.dart';
import 'package:arbiter/features/identity/pk_manager.dart';
import 'package:arbiter/proto/arbiter.pbgrpc.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:cryptography/cryptography.dart';
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
    _tx.add(request);
  }

  Future<UserAgentResponse> receive() async {
    final hasValue = await _rx.moveNext();
    if (!hasValue) {
      throw Exception('Connection closed while waiting for server response.');
    }
    return _rx.current;
  }

  Future<void> close() async {
    await _tx.close();
    await channel.shutdown();
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

List<int> formatChallenge(AuthChallenge challenge, List<int> pubkey) {
  final encodedPubkey = base64Encode(pubkey);
  final payload = "${challenge.nonce}:$encodedPubkey";
  return utf8.encode(payload);
}

const _vaultKeyAssociatedData = 'arbiter.vault.password';

Future<BootstrapResult> bootstrapVault(
  Connection connection,
  String password,
) async {
  final encryptedKey = await _encryptVaultKeyMaterial(connection, password);

  await connection.send(
    UserAgentRequest(
      bootstrapEncryptedKey: BootstrapEncryptedKey(
        nonce: encryptedKey.nonce,
        ciphertext: encryptedKey.ciphertext,
        associatedData: encryptedKey.associatedData,
      ),
    ),
  );

  final response = await connection.receive();
  if (!response.hasBootstrapResult()) {
    throw Exception(
      'Expected bootstrap result, got ${response.whichPayload()}',
    );
  }

  return response.bootstrapResult;
}

Future<UnsealResult> unsealVault(Connection connection, String password) async {
  final encryptedKey = await _encryptVaultKeyMaterial(connection, password);

  await connection.send(
    UserAgentRequest(
      unsealEncryptedKey: UnsealEncryptedKey(
        nonce: encryptedKey.nonce,
        ciphertext: encryptedKey.ciphertext,
        associatedData: encryptedKey.associatedData,
      ),
    ),
  );

  final response = await connection.receive();
  if (!response.hasUnsealResult()) {
    throw Exception('Expected unseal result, got ${response.whichPayload()}');
  }

  return response.unsealResult;
}

Future<_EncryptedVaultKey> _encryptVaultKeyMaterial(
  Connection connection,
  String password,
) async {
  final keyExchange = X25519();
  final cipher = Xchacha20.poly1305Aead();
  final clientKeyPair = await keyExchange.newKeyPair();
  final clientPublicKey = await clientKeyPair.extractPublicKey();

  await connection.send(
    UserAgentRequest(
      unsealStart: UnsealStart(clientPubkey: clientPublicKey.bytes),
    ),
  );

  final handshakeResponse = await connection.receive();
  if (!handshakeResponse.hasUnsealStartResponse()) {
    throw Exception(
      'Expected unseal handshake response, got ${handshakeResponse.whichPayload()}',
    );
  }

  final serverPublicKey = SimplePublicKey(
    handshakeResponse.unsealStartResponse.serverPubkey,
    type: KeyPairType.x25519,
  );
  final sharedSecret = await keyExchange.sharedSecretKey(
    keyPair: clientKeyPair,
    remotePublicKey: serverPublicKey,
  );

  final secretBox = await cipher.encrypt(
    utf8.encode(password),
    secretKey: sharedSecret,
    nonce: cipher.newNonce(),
    aad: utf8.encode(_vaultKeyAssociatedData),
  );

  return _EncryptedVaultKey(
    nonce: secretBox.nonce,
    ciphertext: [...secretBox.cipherText, ...secretBox.mac.bytes],
    associatedData: utf8.encode(_vaultKeyAssociatedData),
  );
}

class _EncryptedVaultKey {
  const _EncryptedVaultKey({
    required this.nonce,
    required this.ciphertext,
    required this.associatedData,
  });

  final List<int> nonce;
  final List<int> ciphertext;
  final List<int> associatedData;
}

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
    await connection.send(UserAgentRequest(authChallengeRequest: req));
    talker.info(
      "Sent auth challenge request with pubkey ${base64Encode(pubkey)}",
    );

    final response = await connection.receive();

    talker.info('Received response from server, checking auth flow...');

    if (response.hasAuthOk()) {
      talker.info('Authentication successful, connection established');
      return connection;
    }

    if (!response.hasAuthChallenge()) {
      throw Exception(
        'Expected AuthChallengeResponse, got ${response.whichPayload()}',
      );
    }

    final challenge = formatChallenge(response.authChallenge, pubkey);
    talker.info(
      'Received auth challenge, signing with key ${base64Encode(pubkey)}',
    );

    final signature = await key.sign(challenge);

    final solutionReq = AuthChallengeSolution(signature: signature);
    await connection.send(UserAgentRequest(authChallengeSolution: solutionReq));

    talker.info('Sent auth challenge solution, waiting for server response...');

    final solutionResponse = await connection.receive();

    if (!solutionResponse.hasAuthOk()) {
      throw Exception(
        'Expected AuthChallengeSolutionResponse, got ${solutionResponse.whichPayload()}',
      );
    }

    talker.info('Authentication successful, connection established');

    return connection;
  } catch (e) {
    throw Exception('Failed to connect to server: $e');
  }
}
