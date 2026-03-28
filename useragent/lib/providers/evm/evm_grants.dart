import 'package:arbiter/features/connection/evm/grants.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:mtcore/markettakers.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'evm_grants.freezed.dart';
part 'evm_grants.g.dart';

final createEvmGrantMutation = Mutation<int>();
final revokeEvmGrantMutation = Mutation<void>();

@freezed
abstract class EvmGrantsState with _$EvmGrantsState {
  const EvmGrantsState._();

  const factory EvmGrantsState({
    required List<GrantEntry> grants,
    @Default(false) bool showRevoked,
  }) = _EvmGrantsState;

  bool get revokedFilterBackedByServer => false;
}

@riverpod
class EvmGrants extends _$EvmGrants {
  @override
  Future<EvmGrantsState?> build() async {
    final connection = await ref.watch(connectionManagerProvider.future);
    if (connection == null) {
      return null;
    }

    try {
      final grants = await listEvmGrants(connection);
      return EvmGrantsState(grants: grants);
    } catch (e, st) {
      talker.handle(e, st);
      rethrow;
    }
  }

  void toggleShowRevoked(bool value) {
    final current = state.asData?.value;
    if (current == null) {
      return;
    }
    state = AsyncData(current.copyWith(showRevoked: value));
  }

  Future<void> refresh() async {
    final connection = await ref.read(connectionManagerProvider.future);
    if (connection == null) {
      state = const AsyncData(null);
      return;
    }

    final previous = state.asData?.value;
    state = const AsyncLoading();

    state = await AsyncValue.guard(() async {
      final grants = await listEvmGrants(connection);
      return EvmGrantsState(
        grants: grants,
        showRevoked: previous?.showRevoked ?? false,
      );
    });
  }
}

Future<int> executeCreateEvmGrant(
  MutationTarget ref, {
  required SharedSettings sharedSettings,
  required SpecificGrant specific,
}) {
  return createEvmGrantMutation.run(ref, (tsx) async {
    final connection = await tsx.get(connectionManagerProvider.future);
    if (connection == null) {
      throw Exception('Not connected to the server.');
    }

    final grantId = await createEvmGrant(
      connection,
      sharedSettings: sharedSettings,
      specific: specific,
    );

    await tsx.get(evmGrantsProvider.notifier).refresh();
    return grantId;
  });
}

Future<void> executeRevokeEvmGrant(MutationTarget ref, {required int grantId}) {
  return revokeEvmGrantMutation.run(ref, (tsx) async {
    final connection = await tsx.get(connectionManagerProvider.future);
    if (connection == null) {
      throw Exception('Not connected to the server.');
    }

    await deleteEvmGrant(connection, grantId);
    await tsx.get(evmGrantsProvider.notifier).refresh();
  });
}
