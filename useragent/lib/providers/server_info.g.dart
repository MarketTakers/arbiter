// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'server_info.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(serverInfoStorage)
final serverInfoStorageProvider = ServerInfoStorageProvider._();

final class ServerInfoStorageProvider
    extends
        $FunctionalProvider<
          ServerInfoStorage,
          ServerInfoStorage,
          ServerInfoStorage
        >
    with $Provider<ServerInfoStorage> {
  ServerInfoStorageProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'serverInfoStorageProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$serverInfoStorageHash();

  @$internal
  @override
  $ProviderElement<ServerInfoStorage> $createElement(
    $ProviderPointer pointer,
  ) => $ProviderElement(pointer);

  @override
  ServerInfoStorage create(Ref ref) {
    return serverInfoStorage(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ServerInfoStorage value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ServerInfoStorage>(value),
    );
  }
}

String _$serverInfoStorageHash() => r'fc06865e7314b1a2493c5de1a9347923a3d21c5c';

@ProviderFor(ServerInfo)
final serverInfoProvider = ServerInfoProvider._();

final class ServerInfoProvider
    extends $AsyncNotifierProvider<ServerInfo, StoredServerInfo?> {
  ServerInfoProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'serverInfoProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$serverInfoHash();

  @$internal
  @override
  ServerInfo create() => ServerInfo();
}

String _$serverInfoHash() => r'6e94f52de03259695a2166b766004eec60ff45fa';

abstract class _$ServerInfo extends $AsyncNotifier<StoredServerInfo?> {
  FutureOr<StoredServerInfo?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref as $Ref<AsyncValue<StoredServerInfo?>, StoredServerInfo?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<StoredServerInfo?>, StoredServerInfo?>,
              AsyncValue<StoredServerInfo?>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
