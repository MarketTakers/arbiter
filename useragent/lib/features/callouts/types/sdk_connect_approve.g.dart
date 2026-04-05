// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'sdk_connect_approve.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(connectApproveEvents)
final connectApproveEventsProvider = ConnectApproveEventsProvider._();

final class ConnectApproveEventsProvider
    extends
        $FunctionalProvider<
          AsyncValue<CalloutEvent>,
          CalloutEvent,
          Stream<CalloutEvent>
        >
    with $FutureModifier<CalloutEvent>, $StreamProvider<CalloutEvent> {
  ConnectApproveEventsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'connectApproveEventsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$connectApproveEventsHash();

  @$internal
  @override
  $StreamProviderElement<CalloutEvent> $createElement(
    $ProviderPointer pointer,
  ) => $StreamProviderElement(pointer);

  @override
  Stream<CalloutEvent> create(Ref ref) {
    return connectApproveEvents(ref);
  }
}

String _$connectApproveEventsHash() =>
    r'abab87cc875a9a4834f836c2c0eba4aa7671d82e';
