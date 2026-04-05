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
          AsyncValue<List<ua_sdk.WalletAccessEntry>?>,
          List<ua_sdk.WalletAccessEntry>?,
          FutureOr<List<ua_sdk.WalletAccessEntry>?>
        >
    with
        $FutureModifier<List<ua_sdk.WalletAccessEntry>?>,
        $FutureProvider<List<ua_sdk.WalletAccessEntry>?> {
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
  $FutureProviderElement<List<ua_sdk.WalletAccessEntry>?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<ua_sdk.WalletAccessEntry>?> create(Ref ref) {
    return walletAccessList(ref);
  }
}

String _$walletAccessListHash() => r'143387471489ebc36de76b2a8ddcb6d857cbad17';
