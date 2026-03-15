import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class StoredServerInfo {
  const StoredServerInfo({
    required this.address,
    required this.port,
    required this.caCertFingerprint,
  });

  final String address;
  final int port;
  final String caCertFingerprint;

  Map<String, dynamic> toJson() => {
    'address': address,
    'port': port,
    'caCertFingerprint': caCertFingerprint,
  };

  factory StoredServerInfo.fromJson(Map<String, dynamic> json) {
    return StoredServerInfo(
      address: json['address'] as String,
      port: json['port'] as int,
      caCertFingerprint: json['caCertFingerprint'] as String,
    );
  }
}

abstract class ServerInfoStorage {
  Future<StoredServerInfo?> load();
  Future<void> save(StoredServerInfo serverInfo);
  Future<void> clear();
}

class SecureServerInfoStorage implements ServerInfoStorage {
  static const _storageKey = 'server_info';

  const SecureServerInfoStorage();

  static const _storage = FlutterSecureStorage();

  @override
  Future<StoredServerInfo?> load() async {
    final rawValue = await _storage.read(key: _storageKey);
    if (rawValue == null) {
      return null;
    }

    final decoded = jsonDecode(rawValue) as Map<String, dynamic>;
    return StoredServerInfo.fromJson(decoded);
  }

  @override
  Future<void> save(StoredServerInfo serverInfo) {
    return _storage.write(
      key: _storageKey,
      value: jsonEncode(serverInfo.toJson()),
    );
  }

  @override
  Future<void> clear() {
    return _storage.delete(key: _storageKey);
  }
}
