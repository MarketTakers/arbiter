// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'vault_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(vaultState)
final vaultStateProvider = VaultStateProvider._();

final class VaultStateProvider
    extends
        $FunctionalProvider<
          AsyncValue<VaultState?>,
          VaultState?,
          FutureOr<VaultState?>
        >
    with $FutureModifier<VaultState?>, $FutureProvider<VaultState?> {
  VaultStateProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'vaultStateProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$vaultStateHash();

  @$internal
  @override
  $FutureProviderElement<VaultState?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<VaultState?> create(Ref ref) {
    return vaultState(ref);
  }
}

String _$vaultStateHash() => r'f7247826d92ed583c475dd7f956b1ffea1f9a7da';
