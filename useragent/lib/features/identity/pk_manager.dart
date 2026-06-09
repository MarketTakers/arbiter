// The API to handle without storing the private key in memory.
//The implementation will use platform-specific secure storage and signing capabilities.
abstract class KeyHandle {
  Future<List<int>> sign(List<int> data);
  Future<List<int>> getPublicKey();
}

abstract class KeyManager {
  Future<KeyHandle?> get();
  Future<KeyHandle> create();
}
