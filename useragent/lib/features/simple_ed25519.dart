import 'dart:convert';

import 'package:cryptography/cryptography.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:arbiter/features/pk_manager.dart';

final storage = FlutterSecureStorage(
  aOptions: AndroidOptions.biometric(
    enforceBiometrics: true,
    biometricPromptTitle: 'Authentication Required',
  ),
  mOptions: MacOsOptions(
    accessibility: KeychainAccessibility.unlocked_this_device,
    label: "Arbiter",
    description: "Confirm your identity to access vault",
    synchronizable: false,
    accessControlFlags: [
      AccessControlFlag.userPresence,
      AccessControlFlag.privateKeyUsage,
    ],
    usesDataProtectionKeychain: true,
  ),
);

final processor = Ed25519();

class SimpleEd25519 extends KeyHandle {
  final SimpleKeyPair _keyPair;

  SimpleEd25519({required SimpleKeyPair keyPair}) : _keyPair = keyPair;

  @override
  KeyAlgorithm get alg => KeyAlgorithm.ed25519;

  @override
  Future<List<int>> getPublicKey() async {
    final publicKey = await _keyPair.extractPublicKey();
    return publicKey.bytes;
  }

  @override
  Future<List<int>> sign(List<int> data) async {
    final signature = await processor.sign(data, keyPair: _keyPair);
    return signature.bytes;
  }
}

class SimpleEd25519Manager extends KeyManager {
  static const _storageKey = "ed25519_identity";
  static const _storagePublicKey = "ed25519_public_key";

  @override
  Future<KeyHandle> create() async {
    final storedKey = await get();
    if (storedKey != null) {
      return storedKey;
    }

    final newKey = await processor.newKeyPair();
    final rawKey = await newKey.extract();

    final keyData = base64Encode(rawKey.bytes);
    await storage.write(key: _storageKey, value: keyData);

    final publicKeyData = base64Encode(rawKey.publicKey.bytes);
    await storage.write(key: _storagePublicKey, value: publicKeyData);

    return SimpleEd25519(keyPair: newKey);
  }

  @override
  Future<KeyHandle?> get() async {
    final storedKeyPair = await storage.read(key: _storageKey);
    if (storedKeyPair == null) {
      return null;
    }

    final publicKeyData = await storage.read(key: _storagePublicKey);
    final publicKeyRaw = base64Decode(publicKeyData!);
    final publicKey = SimplePublicKey(
      publicKeyRaw,
      type: processor.keyPairType,
    );

    final keyBytes = base64Decode(storedKeyPair);
    final keypair = SimpleKeyPairData(
      keyBytes,
      publicKey: publicKey,
      type: processor.keyPairType,
    );

    return SimpleEd25519(keyPair: keypair);
  }
}
