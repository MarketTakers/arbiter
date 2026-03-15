

import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'bootstrap_token.g.dart';

@Riverpod(keepAlive: true)
class BootstrapToken extends _$BootstrapToken {
  String? build() {
    return null;
  }

  void set(String token) {
    state = token;
  }

  String? take() {
    final token = state;
    state = null;
    return token;
  }
}