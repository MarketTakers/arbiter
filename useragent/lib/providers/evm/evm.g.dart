// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'evm.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(Evm)
final evmProvider = EvmProvider._();

final class EvmProvider
    extends $AsyncNotifierProvider<Evm, List<WalletEntry>?> {
  EvmProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'evmProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$evmHash();

  @$internal
  @override
  Evm create() => Evm();
}

String _$evmHash() => r'ca2c9736065c5dc7cc45d8485000dd85dfbfa572';

abstract class _$Evm extends $AsyncNotifier<List<WalletEntry>?> {
  FutureOr<List<WalletEntry>?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref as $Ref<AsyncValue<List<WalletEntry>?>, List<WalletEntry>?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<List<WalletEntry>?>, List<WalletEntry>?>,
              AsyncValue<List<WalletEntry>?>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
