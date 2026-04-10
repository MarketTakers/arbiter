import 'package:arbiter/features/connection/auth.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('certificate pinning helpers', () {
    test('certificateFingerprintHex returns SHA-256 in hex', () {
      final fingerprint = certificateFingerprintHex('abc'.codeUnits);

      expect(
        fingerprint,
        'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
      );
    });

    test('isPinnedServerCertificate matches expected fingerprint', () {
      final matches = isPinnedServerCertificate(
        expectedFingerprint:
            'BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD',
        certificateDer: 'abc'.codeUnits,
      );

      expect(matches, isTrue);
    });

    test('isPinnedServerCertificate rejects mismatched fingerprint', () {
      final matches = isPinnedServerCertificate(
        expectedFingerprint:
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        certificateDer: 'abc'.codeUnits,
      );

      expect(matches, isFalse);
    });
  });
}
