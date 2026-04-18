import 'package:arbiter/features/connection/server_info_storage.dart';
import 'package:cryptography/cryptography.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'server_info.g.dart';

@riverpod
ServerInfoStorage serverInfoStorage(Ref ref) {
  return const SecureServerInfoStorage();
}

@Riverpod(keepAlive: true)
class ServerInfo extends _$ServerInfo {
  @override
  Future<StoredServerInfo?> build() {
    final storage = ref.watch(serverInfoStorageProvider);
    return storage.load();
  }

  Future<void> save({
    required String address,
    required int port,
    required List<int> caCert,
  }) async {
    final storage = ref.read(serverInfoStorageProvider);
    final fingerprint = await _fingerprint(caCert);
    final serverInfo = StoredServerInfo(
      address: address,
      port: port,
      caCertFingerprint: fingerprint,
    );

    state = await AsyncValue.guard(() async {
      await storage.save(serverInfo);
      return serverInfo;
    });
  }

  Future<void> clear() async {
    final storage = ref.read(serverInfoStorageProvider);
    state = await AsyncValue.guard(() async {
      await storage.clear();
      return null;
    });
  }

  Future<String> _fingerprint(List<int> caCert) async {
    final digest = await Sha256().hash(caCert);
    return digest.bytes
        .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
        .join();
  }
}
