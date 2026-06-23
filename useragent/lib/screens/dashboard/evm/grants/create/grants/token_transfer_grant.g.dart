// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'token_transfer_grant.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(TokenGrantLimits)
final tokenGrantLimitsProvider = TokenGrantLimitsProvider._();

final class TokenGrantLimitsProvider
    extends $NotifierProvider<TokenGrantLimits, List<VolumeLimitEntry>> {
  TokenGrantLimitsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'tokenGrantLimitsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$tokenGrantLimitsHash();

  @$internal
  @override
  TokenGrantLimits create() => TokenGrantLimits();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(List<VolumeLimitEntry> value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<List<VolumeLimitEntry>>(value),
    );
  }
}

String _$tokenGrantLimitsHash() => r'84db377f24940d215af82052e27863ab40c02b24';

abstract class _$TokenGrantLimits extends $Notifier<List<VolumeLimitEntry>> {
  List<VolumeLimitEntry> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref as $Ref<List<VolumeLimitEntry>, List<VolumeLimitEntry>>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<List<VolumeLimitEntry>, List<VolumeLimitEntry>>,
              List<VolumeLimitEntry>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
