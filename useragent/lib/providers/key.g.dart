// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'key.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(keyManager)
final keyManagerProvider = KeyManagerProvider._();

final class KeyManagerProvider
    extends $FunctionalProvider<KeyManager, KeyManager, KeyManager>
    with $Provider<KeyManager> {
  KeyManagerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'keyManagerProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$keyManagerHash();

  @$internal
  @override
  $ProviderElement<KeyManager> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  KeyManager create(Ref ref) {
    return keyManager(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(KeyManager value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<KeyManager>(value),
    );
  }
}

String _$keyManagerHash() => r'aa37bca34c01a39c11e29d57e320172b37c0b116';

@ProviderFor(Key)
final keyProvider = KeyProvider._();

final class KeyProvider extends $AsyncNotifierProvider<Key, KeyHandle?> {
  KeyProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'keyProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$keyHash();

  @$internal
  @override
  Key create() => Key();
}

String _$keyHash() => r'6d66204174c4d2d5c76e27f3a8de8f9a9c88a3e0';

abstract class _$Key extends $AsyncNotifier<KeyHandle?> {
  FutureOr<KeyHandle?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<AsyncValue<KeyHandle?>, KeyHandle?>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<AsyncValue<KeyHandle?>, KeyHandle?>,
              AsyncValue<KeyHandle?>,
              Object?,
              Object?
            >;
    element.handleCreate(ref, build);
  }
}
