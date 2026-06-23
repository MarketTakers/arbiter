// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'callout_manager.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(CalloutManager)
final calloutManagerProvider = CalloutManagerProvider._();

final class CalloutManagerProvider
    extends $NotifierProvider<CalloutManager, Map<String, ActiveCallout>> {
  CalloutManagerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'calloutManagerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$calloutManagerHash();

  @$internal
  @override
  CalloutManager create() => CalloutManager();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Map<String, ActiveCallout> value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Map<String, ActiveCallout>>(value),
    );
  }
}

String _$calloutManagerHash() => r'ff8c9a03a6bbbca822242eb497c503b18240a289';

abstract class _$CalloutManager extends $Notifier<Map<String, ActiveCallout>> {
  Map<String, ActiveCallout> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref
            as $Ref<Map<String, ActiveCallout>, Map<String, ActiveCallout>>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<
                Map<String, ActiveCallout>,
                Map<String, ActiveCallout>
              >,
              Map<String, ActiveCallout>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
