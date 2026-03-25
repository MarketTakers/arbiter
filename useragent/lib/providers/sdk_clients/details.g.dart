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
          AsyncValue<SdkClientEntry?>,
          SdkClientEntry?,
          FutureOr<SdkClientEntry?>
        >
    with $FutureModifier<SdkClientEntry?>, $FutureProvider<SdkClientEntry?> {
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
  $FutureProviderElement<SdkClientEntry?> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<SdkClientEntry?> create(Ref ref) {
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

String _$clientDetailsHash() => r'21449a1a2cc4fa4e65ce761e6342e97c1d957a7a';

final class ClientDetailsFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<SdkClientEntry?>, int> {
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
