import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:cryptography/cryptography.dart';

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
    UserAgentRequest(unsealStart: UnsealStart(clientPubkey: clientPublicKey.bytes)),
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
    password.codeUnits,
    secretKey: sharedSecret,
    nonce: cipher.newNonce(),
    aad: _vaultKeyAssociatedData.codeUnits,
  );

  return _EncryptedVaultKey(
    nonce: secretBox.nonce,
    ciphertext: [...secretBox.cipherText, ...secretBox.mac.bytes],
    associatedData: _vaultKeyAssociatedData.codeUnits,
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
