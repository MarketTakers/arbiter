import 'package:arbiter/features/connection/evm/wallet_access.dart';
import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:mtcore/markettakers.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'wallet_access_list.g.dart';

@riverpod
Future<List<ua_sdk.WalletAccessEntry>?> walletAccessList(Ref ref) async {
  final connection = await ref.watch(connectionManagerProvider.future);
  if (connection == null) {
    return null;
  }

  try {
    return await listAllWalletAccesses(connection);
  } catch (e, st) {
    talker.handle(e, st);
    rethrow;
  }
}
