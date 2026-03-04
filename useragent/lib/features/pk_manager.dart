
import 'package:flutter/services.dart';

enum KeyAlgorithm {
  rsa, ecdsa, ed25519
}

// The API to handle without storing the private key in memory. 
//The implementation will use platform-specific secure storage and signing capabilities.
abstract class KeyHandle {
  KeyAlgorithm get alg;
  Future<List<int>> sign(List<int> data);
  Future<List<int>> getPublicKey();
}

abstract class KeyManager {
  Future<KeyHandle?> get();
  Future<KeyHandle> create();
}