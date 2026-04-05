import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/user_agent/vault/bootstrap.pb.dart' as ua_bootstrap;
import 'package:arbiter/proto/user_agent/vault/unseal.pb.dart' as ua_unseal;
import 'package:arbiter/proto/user_agent/vault/vault.pb.dart' as ua_vault;
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:cryptography/cryptography.dart';

const _vaultKeyAssociatedData = 'arbiter.vault.password';

Future<ua_bootstrap.BootstrapResult> bootstrapVault(
  Connection connection,
  String password,
) async {
  final encryptedKey = await _encryptVaultKeyMaterial(connection, password);

  final response = await connection.ask(
    UserAgentRequest(
      vault: ua_vault.Request(
        bootstrap: ua_bootstrap.Request(
          encryptedKey: ua_bootstrap.BootstrapEncryptedKey(
            nonce: encryptedKey.nonce,
            ciphertext: encryptedKey.ciphertext,
            associatedData: encryptedKey.associatedData,
          ),
        ),
      ),
    ),
  );
  if (!response.hasVault()) {
    throw Exception(
      'Expected vault response, got ${response.whichPayload()}',
    );
  }

  final vaultResponse = response.vault;
  if (!vaultResponse.hasBootstrap()) {
    throw Exception(
      'Expected bootstrap result, got ${vaultResponse.whichPayload()}',
    );
  }

  final bootstrapResponse = vaultResponse.bootstrap;
  if (!bootstrapResponse.hasResult()) {
    throw Exception('Expected bootstrap result payload.');
  }

  return bootstrapResponse.result;
}

Future<ua_unseal.UnsealResult> unsealVault(
  Connection connection,
  String password,
) async {
  final encryptedKey = await _encryptVaultKeyMaterial(connection, password);

  final response = await connection.ask(
    UserAgentRequest(
      vault: ua_vault.Request(
        unseal: ua_unseal.Request(
          encryptedKey: ua_unseal.UnsealEncryptedKey(
            nonce: encryptedKey.nonce,
            ciphertext: encryptedKey.ciphertext,
            associatedData: encryptedKey.associatedData,
          ),
        ),
      ),
    ),
  );
  if (!response.hasVault()) {
    throw Exception('Expected vault response, got ${response.whichPayload()}');
  }

  final vaultResponse = response.vault;
  if (!vaultResponse.hasUnseal()) {
    throw Exception(
      'Expected unseal result, got ${vaultResponse.whichPayload()}',
    );
  }

  final unsealResponse = vaultResponse.unseal;
  if (!unsealResponse.hasResult()) {
    throw Exception(
      'Expected unseal result payload, got ${unsealResponse.whichPayload()}',
    );
  }

  return unsealResponse.result;
}

Future<_EncryptedVaultKey> _encryptVaultKeyMaterial(
  Connection connection,
  String password,
) async {
  final keyExchange = X25519();
  final cipher = Xchacha20.poly1305Aead();
  final clientKeyPair = await keyExchange.newKeyPair();
  final clientPublicKey = await clientKeyPair.extractPublicKey();

  final handshakeResponse = await connection.ask(
    UserAgentRequest(
      vault: ua_vault.Request(
        unseal: ua_unseal.Request(
          start: ua_unseal.UnsealStart(clientPubkey: clientPublicKey.bytes),
        ),
      ),
    ),
  );
  if (!handshakeResponse.hasVault()) {
    throw Exception(
      'Expected vault response, got ${handshakeResponse.whichPayload()}',
    );
  }

  final vaultResponse = handshakeResponse.vault;
  if (!vaultResponse.hasUnseal()) {
    throw Exception(
      'Expected unseal handshake response, got ${vaultResponse.whichPayload()}',
    );
  }

  final unsealResponse = vaultResponse.unseal;
  if (!unsealResponse.hasStart()) {
    throw Exception(
      'Expected unseal handshake payload, got ${unsealResponse.whichPayload()}',
    );
  }

  final serverPublicKey = SimplePublicKey(
    unsealResponse.start.serverPubkey,
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
