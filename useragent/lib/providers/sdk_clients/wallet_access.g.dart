// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_access.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(walletAccess)
final walletAccessProvider = WalletAccessProvider._();

final class WalletAccessProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<SdkClientWalletAccess>?>,
          List<SdkClientWalletAccess>?,
          FutureOr<List<SdkClientWalletAccess>?>
        >
    with
        $FutureModifier<List<SdkClientWalletAccess>?>,
        $FutureProvider<List<SdkClientWalletAccess>?> {
  WalletAccessProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'walletAccessProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$walletAccessHash();

  @$internal
  @override
  $FutureProviderElement<List<SdkClientWalletAccess>?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<SdkClientWalletAccess>?> create(Ref ref) {
    return walletAccess(ref);
  }
}

String _$walletAccessHash() => r'954aae12d2d18999efaa97d01be983bf849f2296';
