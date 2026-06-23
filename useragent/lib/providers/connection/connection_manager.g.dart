// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'connection_manager.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ConnectionManager)
final connectionManagerProvider = ConnectionManagerProvider._();

final class ConnectionManagerProvider
    extends $AsyncNotifierProvider<ConnectionManager, Connection?> {
  ConnectionManagerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'connectionManagerProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$connectionManagerHash();

  @$internal
  @override
  ConnectionManager create() => ConnectionManager();
}

String _$connectionManagerHash() => r'f471afb49bdcde77238424942f5af1716634f084';

abstract class _$ConnectionManager extends $AsyncNotifier<Connection?> {
  FutureOr<Connection?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<AsyncValue<Connection?>, Connection?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<Connection?>, Connection?>,
              AsyncValue<Connection?>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
