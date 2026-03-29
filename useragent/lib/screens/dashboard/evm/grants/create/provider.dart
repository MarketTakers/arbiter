import 'package:arbiter/proto/evm.pb.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'provider.freezed.dart';
part 'provider.g.dart';

@freezed
abstract class GrantCreationState with _$GrantCreationState {
  const factory GrantCreationState({
    int? selectedClientId,
    @Default(SpecificGrant_Grant.etherTransfer) SpecificGrant_Grant grantType,
  }) = _GrantCreationState;
}

@riverpod
class GrantCreation extends _$GrantCreation {
  @override
  GrantCreationState build() => const GrantCreationState();

  void setClientId(int? id) => state = state.copyWith(selectedClientId: id);
  void setGrantType(SpecificGrant_Grant type) =>
      state = state.copyWith(grantType: type);
}
