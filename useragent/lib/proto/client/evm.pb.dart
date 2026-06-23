// This is a generated file - do not edit.
//
// Generated from client/evm.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../evm.pb.dart' as $0;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

enum Request_Payload { signTransaction, analyzeTransaction, notSet }

class Request extends $pb.GeneratedMessage {
  factory Request({
    $0.EvmSignTransactionRequest? signTransaction,
    $0.EvmAnalyzeTransactionRequest? analyzeTransaction,
  }) {
    final result = create();
    if (signTransaction != null) result.signTransaction = signTransaction;
    if (analyzeTransaction != null)
      result.analyzeTransaction = analyzeTransaction;
    return result;
  }

  Request._();

  factory Request.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Request.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Request_Payload> _Request_PayloadByTag = {
    1: Request_Payload.signTransaction,
    2: Request_Payload.analyzeTransaction,
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<$0.EvmSignTransactionRequest>(
        1, _omitFieldNames ? '' : 'signTransaction',
        subBuilder: $0.EvmSignTransactionRequest.create)
    ..aOM<$0.EvmAnalyzeTransactionRequest>(
        2, _omitFieldNames ? '' : 'analyzeTransaction',
        subBuilder: $0.EvmAnalyzeTransactionRequest.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Request clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Request copyWith(void Function(Request) updates) =>
      super.copyWith((message) => updates(message as Request)) as Request;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Request create() => Request._();
  @$core.override
  Request createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Request getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Request>(create);
  static Request? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  Request_Payload whichPayload() => _Request_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $0.EvmSignTransactionRequest get signTransaction => $_getN(0);
  @$pb.TagNumber(1)
  set signTransaction($0.EvmSignTransactionRequest value) =>
      $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSignTransaction() => $_has(0);
  @$pb.TagNumber(1)
  void clearSignTransaction() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EvmSignTransactionRequest ensureSignTransaction() => $_ensure(0);

  @$pb.TagNumber(2)
  $0.EvmAnalyzeTransactionRequest get analyzeTransaction => $_getN(1);
  @$pb.TagNumber(2)
  set analyzeTransaction($0.EvmAnalyzeTransactionRequest value) =>
      $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAnalyzeTransaction() => $_has(1);
  @$pb.TagNumber(2)
  void clearAnalyzeTransaction() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.EvmAnalyzeTransactionRequest ensureAnalyzeTransaction() => $_ensure(1);
}

enum Response_Payload { signTransaction, analyzeTransaction, notSet }

class Response extends $pb.GeneratedMessage {
  factory Response({
    $0.EvmSignTransactionResponse? signTransaction,
    $0.EvmAnalyzeTransactionResponse? analyzeTransaction,
  }) {
    final result = create();
    if (signTransaction != null) result.signTransaction = signTransaction;
    if (analyzeTransaction != null)
      result.analyzeTransaction = analyzeTransaction;
    return result;
  }

  Response._();

  factory Response.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Response.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Response_Payload> _Response_PayloadByTag = {
    1: Response_Payload.signTransaction,
    2: Response_Payload.analyzeTransaction,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<$0.EvmSignTransactionResponse>(
        1, _omitFieldNames ? '' : 'signTransaction',
        subBuilder: $0.EvmSignTransactionResponse.create)
    ..aOM<$0.EvmAnalyzeTransactionResponse>(
        2, _omitFieldNames ? '' : 'analyzeTransaction',
        subBuilder: $0.EvmAnalyzeTransactionResponse.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Response clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Response copyWith(void Function(Response) updates) =>
      super.copyWith((message) => updates(message as Response)) as Response;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Response create() => Response._();
  @$core.override
  Response createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Response getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Response>(create);
  static Response? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $0.EvmSignTransactionResponse get signTransaction => $_getN(0);
  @$pb.TagNumber(1)
  set signTransaction($0.EvmSignTransactionResponse value) =>
      $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSignTransaction() => $_has(0);
  @$pb.TagNumber(1)
  void clearSignTransaction() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EvmSignTransactionResponse ensureSignTransaction() => $_ensure(0);

  @$pb.TagNumber(2)
  $0.EvmAnalyzeTransactionResponse get analyzeTransaction => $_getN(1);
  @$pb.TagNumber(2)
  set analyzeTransaction($0.EvmAnalyzeTransactionResponse value) =>
      $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAnalyzeTransaction() => $_has(1);
  @$pb.TagNumber(2)
  void clearAnalyzeTransaction() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.EvmAnalyzeTransactionResponse ensureAnalyzeTransaction() => $_ensure(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
