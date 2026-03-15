import 'dart:convert';

class ArbiterUrl {
  const ArbiterUrl({
    required this.host,
    required this.port,
    required this.caCert,
    this.bootstrapToken,
  });

  final String host;
  final int port;
  final List<int> caCert;
  final String? bootstrapToken;

  static const _scheme = 'arbiter';
  static const _certQueryKey = 'cert';
  static const _bootstrapTokenQueryKey = 'bootstrap_token';

  static ArbiterUrl parse(String value) {
    final uri = Uri.tryParse(value);
    if (uri == null || uri.scheme != _scheme) {
      throw const FormatException("Invalid URL scheme, expected 'arbiter://'");
    }

    if (uri.host.isEmpty) {
      throw const FormatException('Missing host in URL');
    }

    if (!uri.hasPort) {
      throw const FormatException('Missing port in URL');
    }

    final cert = uri.queryParameters[_certQueryKey];
    if (cert == null || cert.isEmpty) {
      throw const FormatException("Missing 'cert' query parameter in URL");
    }

    final decodedCert = _decodeCert(cert);

    return ArbiterUrl(
      host: uri.host,
      port: uri.port,
      caCert: decodedCert,
      bootstrapToken: uri.queryParameters[_bootstrapTokenQueryKey],
    );
  }

  static List<int> _decodeCert(String cert) {
    try {
      return base64Url.decode(base64Url.normalize(cert));
    } on FormatException catch (error) {
      throw FormatException("Invalid base64 in 'cert' query parameter: ${error.message}");
    }
  }
}
