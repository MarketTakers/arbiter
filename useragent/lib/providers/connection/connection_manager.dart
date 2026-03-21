import 'package:arbiter/features/connection/auth.dart';
import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/providers/connection/bootstrap_token.dart';
import 'package:arbiter/providers/key.dart';
import 'package:arbiter/providers/server_info.dart';
import 'package:mtcore/markettakers.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'connection_manager.g.dart';

@Riverpod(keepAlive: true)
class ConnectionManager extends _$ConnectionManager {
  @override
  Future<Connection?> build() async {
    final serverInfo = await ref.watch(serverInfoProvider.future);
    final key = await ref.watch(keyProvider.future);
    final token = ref.watch(bootstrapTokenProvider);

    if (serverInfo == null || key == null) {
      return null;
    }
    final Connection connection;
    try {
      connection = await connectAndAuthorize(
        serverInfo,
        key,
        bootstrapToken: token,
      );
      if (token != null) {
        ref.read(bootstrapTokenProvider.notifier).clear();
      }
    } catch (e) {
      talker.handle(e);
      rethrow;
    }

    ref.onDispose(() {
      final connection = state.asData?.value;
      if (connection != null) {
        connection.close();
      }
    });

    return connection;
  }
}
