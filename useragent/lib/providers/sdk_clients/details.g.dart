// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'details.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(clientDetails)
final clientDetailsProvider = ClientDetailsFamily._();

final class ClientDetailsProvider
    extends
        $FunctionalProvider<
          AsyncValue<ua_sdk.Entry?>,
          ua_sdk.Entry?,
          FutureOr<ua_sdk.Entry?>
        >
    with $FutureModifier<ua_sdk.Entry?>, $FutureProvider<ua_sdk.Entry?> {
  ClientDetailsProvider._({
    required ClientDetailsFamily super.from,
    required int super.argument,
  }) : super(
         retry: null,
         name: r'clientDetailsProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$clientDetailsHash();

  @override
  String toString() {
    return r'clientDetailsProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<ua_sdk.Entry?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<ua_sdk.Entry?> create(Ref ref) {
    final argument = this.argument as int;
    return clientDetails(ref, argument);
  }

  @override
  bool operator ==(Object other) {
    return other is ClientDetailsProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$clientDetailsHash() => r'907fd39230cc630dcaad3bbe924f343a84a2375e';

final class ClientDetailsFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<ua_sdk.Entry?>, int> {
  ClientDetailsFamily._()
    : super(
        retry: null,
        name: r'clientDetailsProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  ClientDetailsProvider call(int clientId) =>
      ClientDetailsProvider._(argument: clientId, from: this);

  @override
  String toString() => r'clientDetailsProvider';
}
