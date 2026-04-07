import 'dart:convert';

import 'package:arbiter/src/rust/api.dart';
import 'package:cryptography/cryptography.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:arbiter/features/identity/pk_manager.dart';

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
    accessControlFlags: [AccessControlFlag.userPresence],
    usesDataProtectionKeychain: true,
  ),
);

class HazmatMldsa extends KeyHandle {
  final MldsaKey _key;

  HazmatMldsa({required MldsaKey key}) : _key = key;

  @override
  Future<List<int>> getPublicKey() async {
    final publicKey = await _key.getPublicKey();
    return publicKey;
  }

  @override
  Future<List<int>> sign(List<int> data) async {
    final signature = await _key.sign(message: data);
    return signature;
  }
}

class HazmatMLDSAManager extends KeyManager {
  static const _storageKey = "ed25519_identity";

  @override
  Future<KeyHandle> create() async {
    final storedKey = await get();
    if (storedKey != null) {
      return storedKey;
    }

    final newKeypair = await MldsaKey.generate();
    final keyBytes = await newKeypair.toBytes();

    await storage.write(key: _storageKey, value: base64Encode(keyBytes));

    return HazmatMldsa(key: newKeypair);
  }

  @override
  Future<KeyHandle?> get() async {
    final storedKeyPair = await storage.read(key: _storageKey);
    if (storedKeyPair == null) {
      return null;
    }

    final keyBytes = base64Decode(storedKeyPair);
    final key = await MldsaKey.fromBytes(bytes: keyBytes);

    return HazmatMldsa(key: key);
  }
}
