// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'ether_transfer_grant.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(EtherGrantTargets)
final etherGrantTargetsProvider = EtherGrantTargetsProvider._();

final class EtherGrantTargetsProvider
    extends $NotifierProvider<EtherGrantTargets, List<EtherTargetEntry>> {
  EtherGrantTargetsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'etherGrantTargetsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$etherGrantTargetsHash();

  @$internal
  @override
  EtherGrantTargets create() => EtherGrantTargets();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(List<EtherTargetEntry> value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<List<EtherTargetEntry>>(value),
    );
  }
}

String _$etherGrantTargetsHash() => r'063aa3180d5e5bbc1525702272686f1fd8ca751d';

abstract class _$EtherGrantTargets extends $Notifier<List<EtherTargetEntry>> {
  List<EtherTargetEntry> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref as $Ref<List<EtherTargetEntry>, List<EtherTargetEntry>>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<List<EtherTargetEntry>, List<EtherTargetEntry>>,
              List<EtherTargetEntry>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
