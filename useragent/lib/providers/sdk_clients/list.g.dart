// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'list.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(sdkClients)
final sdkClientsProvider = SdkClientsProvider._();

final class SdkClientsProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<SdkClientEntry>?>,
          List<SdkClientEntry>?,
          FutureOr<List<SdkClientEntry>?>
        >
    with
        $FutureModifier<List<SdkClientEntry>?>,
        $FutureProvider<List<SdkClientEntry>?> {
  SdkClientsProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'sdkClientsProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$sdkClientsHash();

  @$internal
  @override
  $FutureProviderElement<List<SdkClientEntry>?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<SdkClientEntry>?> create(Ref ref) {
    return sdkClients(ref);
  }
}

String _$sdkClientsHash() => r'833c249d9cc2f83921453e0ece354a9a2d9f4482';
