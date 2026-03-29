// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wallet_access_list.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(walletAccessList)
final walletAccessListProvider = WalletAccessListProvider._();

final class WalletAccessListProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<SdkClientWalletAccess>?>,
          List<SdkClientWalletAccess>?,
          FutureOr<List<SdkClientWalletAccess>?>
        >
    with
        $FutureModifier<List<SdkClientWalletAccess>?>,
        $FutureProvider<List<SdkClientWalletAccess>?> {
  WalletAccessListProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'walletAccessListProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$walletAccessListHash();

  @$internal
  @override
  $FutureProviderElement<List<SdkClientWalletAccess>?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<SdkClientWalletAccess>?> create(Ref ref) {
    return walletAccessList(ref);
  }
}

String _$walletAccessListHash() => r'c06006d6792ae463105a539723e9bb396192f96b';
