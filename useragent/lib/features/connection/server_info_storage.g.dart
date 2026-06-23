// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'server_info_storage.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

StoredServerInfo _$StoredServerInfoFromJson(Map<String, dynamic> json) =>
    StoredServerInfo(
      address: json['address'] as String,
      port: (json['port'] as num).toInt(),
      caCertFingerprint: json['caCertFingerprint'] as String,
    );

Map<String, dynamic> _$StoredServerInfoToJson(StoredServerInfo instance) =>
    <String, dynamic>{
      'address': instance.address,
      'port': instance.port,
      'caCertFingerprint': instance.caCertFingerprint,
    };
