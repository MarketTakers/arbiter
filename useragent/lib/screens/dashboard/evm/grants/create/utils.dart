import 'package:arbiter/proto/evm.pb.dart';
import 'package:fixnum/fixnum.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

Timestamp toTimestamp(DateTime value) {
  final utc = value.toUtc();
  return Timestamp()
    ..seconds = Int64(utc.millisecondsSinceEpoch ~/ 1000)
    ..nanos = (utc.microsecondsSinceEpoch % 1000000) * 1000;
}

TransactionRateLimit? buildRateLimit(String countText, String windowText) {
  if (countText.trim().isEmpty || windowText.trim().isEmpty) {
    return null;
  }
  return TransactionRateLimit(
    count: int.parse(countText.trim()),
    windowSecs: Int64.parseInt(windowText.trim()),
  );
}

VolumeRateLimit? buildVolumeLimit(String amountText, String windowText) {
  if (amountText.trim().isEmpty || windowText.trim().isEmpty) {
    return null;
  }
  return VolumeRateLimit(
    maxVolume: parseBigIntBytes(amountText),
    windowSecs: Int64.parseInt(windowText.trim()),
  );
}

List<int>? optionalBigIntBytes(String value) {
  if (value.trim().isEmpty) {
    return null;
  }
  return parseBigIntBytes(value);
}

List<int> parseBigIntBytes(String value) {
  final number = BigInt.parse(value.trim());
  if (number < BigInt.zero) {
    throw Exception('Numeric values must be positive.');
  }
  if (number == BigInt.zero) {
    return [0];
  }

  var remaining = number;
  final bytes = <int>[];
  while (remaining > BigInt.zero) {
    bytes.insert(0, (remaining & BigInt.from(0xff)).toInt());
    remaining >>= 8;
  }
  return bytes;
}

List<int> parseHexAddress(String value) {
  final normalized = value.trim().replaceFirst(RegExp(r'^0x'), '');
  if (normalized.length != 40) {
    throw Exception('Expected a 20-byte hex address.');
  }
  return [
    for (var i = 0; i < normalized.length; i += 2)
      int.parse(normalized.substring(i, i + 2), radix: 16),
  ];
}

String shortAddress(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '0x${hex.substring(0, 6)}...${hex.substring(hex.length - 4)}';
}
