// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'evm_grants.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(EvmGrants)
final evmGrantsProvider = EvmGrantsProvider._();

final class EvmGrantsProvider
    extends $AsyncNotifierProvider<EvmGrants, EvmGrantsState?> {
  EvmGrantsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'evmGrantsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$evmGrantsHash();

  @$internal
  @override
  EvmGrants create() => EvmGrants();
}

String _$evmGrantsHash() => r'd71ec12bbc1b412f11fdbaae27382b289f8a3538';

abstract class _$EvmGrants extends $AsyncNotifier<EvmGrantsState?> {
  FutureOr<EvmGrantsState?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<AsyncValue<EvmGrantsState?>, EvmGrantsState?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<EvmGrantsState?>, EvmGrantsState?>,
              AsyncValue<EvmGrantsState?>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
