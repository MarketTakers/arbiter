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
          AsyncValue<List<ua_sdk.Entry>?>,
          List<ua_sdk.Entry>?,
          FutureOr<List<ua_sdk.Entry>?>
        >
    with
        $FutureModifier<List<ua_sdk.Entry>?>,
        $FutureProvider<List<ua_sdk.Entry>?> {
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
  $FutureProviderElement<List<ua_sdk.Entry>?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<ua_sdk.Entry>?> create(Ref ref) {
    return sdkClients(ref);
  }
}

String _$sdkClientsHash() => r'9b966083effea11035d6edde379e71cc2a0f85c0';
