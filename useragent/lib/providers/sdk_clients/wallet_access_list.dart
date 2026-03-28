import 'package:arbiter/features/connection/evm/wallet_access.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:mtcore/markettakers.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'wallet_access_list.g.dart';

@riverpod
Future<List<SdkClientWalletAccess>?> walletAccessList(Ref ref) async {
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
