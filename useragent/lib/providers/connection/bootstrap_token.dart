import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'bootstrap_token.g.dart';

@Riverpod(keepAlive: true)
class BootstrapToken extends _$BootstrapToken {
  @override
  String? build() {
    return null;
  }

  void set(String token) {
    state = token;
  }

  void clear() {
    state = null;
  }

  String? take() {
    final token = state;
    state = null;
    return token;
  }
}
