// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'bootstrap_token.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(BootstrapToken)
final bootstrapTokenProvider = BootstrapTokenProvider._();

final class BootstrapTokenProvider
    extends $NotifierProvider<BootstrapToken, String?> {
  BootstrapTokenProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'bootstrapTokenProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$bootstrapTokenHash();

  @$internal
  @override
  BootstrapToken create() => BootstrapToken();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(String? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<String?>(value),
    );
  }
}

String _$bootstrapTokenHash() => r'a59e679ab0561ed2ab4148660499891571d439db';

abstract class _$BootstrapToken extends $Notifier<String?> {
  String? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<String?, String?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<String?, String?>,
              String?,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
