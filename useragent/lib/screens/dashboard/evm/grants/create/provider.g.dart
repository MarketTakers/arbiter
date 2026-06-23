// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(GrantCreation)
final grantCreationProvider = GrantCreationProvider._();

final class GrantCreationProvider
    extends $NotifierProvider<GrantCreation, GrantCreationState> {
  GrantCreationProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'grantCreationProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$grantCreationHash();

  @$internal
  @override
  GrantCreation create() => GrantCreation();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(GrantCreationState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<GrantCreationState>(value),
    );
  }
}

String _$grantCreationHash() => r'3733d45da30990ef8ecbee946d2eae81bc7f5fc9';

abstract class _$GrantCreation extends $Notifier<GrantCreationState> {
  GrantCreationState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<GrantCreationState, GrantCreationState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<GrantCreationState, GrantCreationState>,
              GrantCreationState,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
