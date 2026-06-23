// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_access.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(clientWalletAccessRepository)
final clientWalletAccessRepositoryProvider =
    ClientWalletAccessRepositoryProvider._();

final class ClientWalletAccessRepositoryProvider
    extends
        $FunctionalProvider<
          ClientWalletAccessRepository,
          ClientWalletAccessRepository,
          ClientWalletAccessRepository
        >
    with $Provider<ClientWalletAccessRepository> {
  ClientWalletAccessRepositoryProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'clientWalletAccessRepositoryProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$clientWalletAccessRepositoryHash();

  @$internal
  @override
  $ProviderElement<ClientWalletAccessRepository> $createElement(
    $ProviderPointer pointer,
  ) => $ProviderElement(pointer);

  @override
  ClientWalletAccessRepository create(Ref ref) {
    return clientWalletAccessRepository(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ClientWalletAccessRepository value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ClientWalletAccessRepository>(value),
    );
  }
}

String _$clientWalletAccessRepositoryHash() =>
    r'bbc332284bc36a8b5d807bd5c45362b6b12b19e7';

@ProviderFor(clientWalletOptions)
final clientWalletOptionsProvider = ClientWalletOptionsProvider._();

final class ClientWalletOptionsProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<ClientWalletOption>>,
          List<ClientWalletOption>,
          FutureOr<List<ClientWalletOption>>
        >
    with
        $FutureModifier<List<ClientWalletOption>>,
        $FutureProvider<List<ClientWalletOption>> {
  ClientWalletOptionsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'clientWalletOptionsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$clientWalletOptionsHash();

  @$internal
  @override
  $FutureProviderElement<List<ClientWalletOption>> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<ClientWalletOption>> create(Ref ref) {
    return clientWalletOptions(ref);
  }
}

String _$clientWalletOptionsHash() =>
    r'32183c2b281e2a41400de07f2381132a706815ab';

@ProviderFor(clientWalletAccessSelection)
final clientWalletAccessSelectionProvider =
    ClientWalletAccessSelectionFamily._();

final class ClientWalletAccessSelectionProvider
    extends
        $FunctionalProvider<AsyncValue<Set<int>>, Set<int>, FutureOr<Set<int>>>
    with $FutureModifier<Set<int>>, $FutureProvider<Set<int>> {
  ClientWalletAccessSelectionProvider._({
    required ClientWalletAccessSelectionFamily super.from,
    required int super.argument,
  }) : super(
         retry: null,
         name: r'clientWalletAccessSelectionProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$clientWalletAccessSelectionHash();

  @override
  String toString() {
    return r'clientWalletAccessSelectionProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<Set<int>> $createElement($ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<Set<int>> create(Ref ref) {
    final argument = this.argument as int;
    return clientWalletAccessSelection(ref, argument);
  }

  @override
  bool operator ==(Object other) {
    return other is ClientWalletAccessSelectionProvider &&
        other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$clientWalletAccessSelectionHash() =>
    r'f33705ee7201cd9b899cc058d6642de85a22b03e';

final class ClientWalletAccessSelectionFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<Set<int>>, int> {
  ClientWalletAccessSelectionFamily._()
    : super(
        retry: null,
        name: r'clientWalletAccessSelectionProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  ClientWalletAccessSelectionProvider call(int clientId) =>
      ClientWalletAccessSelectionProvider._(argument: clientId, from: this);

  @override
  String toString() => r'clientWalletAccessSelectionProvider';
}

@ProviderFor(ClientWalletAccessController)
final clientWalletAccessControllerProvider =
    ClientWalletAccessControllerFamily._();

final class ClientWalletAccessControllerProvider
    extends
        $NotifierProvider<
          ClientWalletAccessController,
          ClientWalletAccessState
        > {
  ClientWalletAccessControllerProvider._({
    required ClientWalletAccessControllerFamily super.from,
    required int super.argument,
  }) : super(
         retry: null,
         name: r'clientWalletAccessControllerProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$clientWalletAccessControllerHash();

  @override
  String toString() {
    return r'clientWalletAccessControllerProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  ClientWalletAccessController create() => ClientWalletAccessController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ClientWalletAccessState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ClientWalletAccessState>(value),
    );
  }

  @override
  bool operator ==(Object other) {
    return other is ClientWalletAccessControllerProvider &&
        other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$clientWalletAccessControllerHash() =>
    r'45bff81382fec3e8610190167b55667a7dfc1111';

final class ClientWalletAccessControllerFamily extends $Family
    with
        $ClassFamilyOverride<
          ClientWalletAccessController,
          ClientWalletAccessState,
          ClientWalletAccessState,
          ClientWalletAccessState,
          int
        > {
  ClientWalletAccessControllerFamily._()
    : super(
        retry: null,
        name: r'clientWalletAccessControllerProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  ClientWalletAccessControllerProvider call(int clientId) =>
      ClientWalletAccessControllerProvider._(argument: clientId, from: this);

  @override
  String toString() => r'clientWalletAccessControllerProvider';
}

abstract class _$ClientWalletAccessController
    extends $Notifier<ClientWalletAccessState> {
  late final _$args = ref.$arg as int;
  int get clientId => _$args;

  ClientWalletAccessState build(int clientId);
  @$mustCallSuper
  @override
  void runBuild() {
    final ref =
        this.ref as $Ref<ClientWalletAccessState, ClientWalletAccessState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ClientWalletAccessState, ClientWalletAccessState>,
              ClientWalletAccessState,
              Object?,
              Object?
            >;
    element.handleCreate(ref, () => build(_$args));
  }
}
