// This is a generated file - do not edit.
//
// Generated from user_agent/vault/unseal.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import 'unseal.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'unseal.pbenum.dart';

class UnsealStart extends $pb.GeneratedMessage {
  factory UnsealStart({
    $core.List<$core.int>? clientPubkey,
  }) {
    final result = create();
    if (clientPubkey != null) result.clientPubkey = clientPubkey;
    return result;
  }

  UnsealStart._();

  factory UnsealStart.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UnsealStart.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UnsealStart',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault.unseal'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'clientPubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnsealStart clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnsealStart copyWith(void Function(UnsealStart) updates) =>
      super.copyWith((message) => updates(message as UnsealStart))
          as UnsealStart;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UnsealStart create() => UnsealStart._();
  @$core.override
  UnsealStart createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UnsealStart getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UnsealStart>(create);
  static UnsealStart? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get clientPubkey => $_getN(0);
  @$pb.TagNumber(1)
  set clientPubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientPubkey() => $_clearField(1);
}

class UnsealStartResponse extends $pb.GeneratedMessage {
  factory UnsealStartResponse({
    $core.List<$core.int>? serverPubkey,
  }) {
    final result = create();
    if (serverPubkey != null) result.serverPubkey = serverPubkey;
    return result;
  }

  UnsealStartResponse._();

  factory UnsealStartResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UnsealStartResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UnsealStartResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault.unseal'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'serverPubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnsealStartResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnsealStartResponse copyWith(void Function(UnsealStartResponse) updates) =>
      super.copyWith((message) => updates(message as UnsealStartResponse))
          as UnsealStartResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UnsealStartResponse create() => UnsealStartResponse._();
  @$core.override
  UnsealStartResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UnsealStartResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UnsealStartResponse>(create);
  static UnsealStartResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get serverPubkey => $_getN(0);
  @$pb.TagNumber(1)
  set serverPubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServerPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearServerPubkey() => $_clearField(1);
}

class UnsealEncryptedKey extends $pb.GeneratedMessage {
  factory UnsealEncryptedKey({
    $core.List<$core.int>? nonce,
    $core.List<$core.int>? ciphertext,
    $core.List<$core.int>? associatedData,
  }) {
    final result = create();
    if (nonce != null) result.nonce = nonce;
    if (ciphertext != null) result.ciphertext = ciphertext;
    if (associatedData != null) result.associatedData = associatedData;
    return result;
  }

  UnsealEncryptedKey._();

  factory UnsealEncryptedKey.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UnsealEncryptedKey.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UnsealEncryptedKey',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault.unseal'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'nonce', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'ciphertext', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'associatedData', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnsealEncryptedKey clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnsealEncryptedKey copyWith(void Function(UnsealEncryptedKey) updates) =>
      super.copyWith((message) => updates(message as UnsealEncryptedKey))
          as UnsealEncryptedKey;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UnsealEncryptedKey create() => UnsealEncryptedKey._();
  @$core.override
  UnsealEncryptedKey createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UnsealEncryptedKey getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UnsealEncryptedKey>(create);
  static UnsealEncryptedKey? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get nonce => $_getN(0);
  @$pb.TagNumber(1)
  set nonce($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNonce() => $_has(0);
  @$pb.TagNumber(1)
  void clearNonce() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get ciphertext => $_getN(1);
  @$pb.TagNumber(2)
  set ciphertext($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCiphertext() => $_has(1);
  @$pb.TagNumber(2)
  void clearCiphertext() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.List<$core.int> get associatedData => $_getN(2);
  @$pb.TagNumber(3)
  set associatedData($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAssociatedData() => $_has(2);
  @$pb.TagNumber(3)
  void clearAssociatedData() => $_clearField(3);
}

enum Request_Payload { start, encryptedKey, notSet }

class Request extends $pb.GeneratedMessage {
  factory Request({
    UnsealStart? start,
    UnsealEncryptedKey? encryptedKey,
  }) {
    final result = create();
    if (start != null) result.start = start;
    if (encryptedKey != null) result.encryptedKey = encryptedKey;
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
    1: Request_Payload.start,
    2: Request_Payload.encryptedKey,
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault.unseal'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<UnsealStart>(1, _omitFieldNames ? '' : 'start',
        subBuilder: UnsealStart.create)
    ..aOM<UnsealEncryptedKey>(2, _omitFieldNames ? '' : 'encryptedKey',
        subBuilder: UnsealEncryptedKey.create)
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
  UnsealStart get start => $_getN(0);
  @$pb.TagNumber(1)
  set start(UnsealStart value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasStart() => $_has(0);
  @$pb.TagNumber(1)
  void clearStart() => $_clearField(1);
  @$pb.TagNumber(1)
  UnsealStart ensureStart() => $_ensure(0);

  @$pb.TagNumber(2)
  UnsealEncryptedKey get encryptedKey => $_getN(1);
  @$pb.TagNumber(2)
  set encryptedKey(UnsealEncryptedKey value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEncryptedKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearEncryptedKey() => $_clearField(2);
  @$pb.TagNumber(2)
  UnsealEncryptedKey ensureEncryptedKey() => $_ensure(1);
}

enum Response_Payload { start, result, notSet }

class Response extends $pb.GeneratedMessage {
  factory Response({
    UnsealStartResponse? start,
    UnsealResult? result,
  }) {
    final result$ = create();
    if (start != null) result$.start = start;
    if (result != null) result$.result = result;
    return result$;
  }

  Response._();

  factory Response.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Response.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Response_Payload> _Response_PayloadByTag = {
    1: Response_Payload.start,
    2: Response_Payload.result,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault.unseal'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<UnsealStartResponse>(1, _omitFieldNames ? '' : 'start',
        subBuilder: UnsealStartResponse.create)
    ..aE<UnsealResult>(2, _omitFieldNames ? '' : 'result',
        enumValues: UnsealResult.values)
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
  UnsealStartResponse get start => $_getN(0);
  @$pb.TagNumber(1)
  set start(UnsealStartResponse value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasStart() => $_has(0);
  @$pb.TagNumber(1)
  void clearStart() => $_clearField(1);
  @$pb.TagNumber(1)
  UnsealStartResponse ensureStart() => $_ensure(0);

  @$pb.TagNumber(2)
  UnsealResult get result => $_getN(1);
  @$pb.TagNumber(2)
  set result(UnsealResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasResult() => $_has(1);
  @$pb.TagNumber(2)
  void clearResult() => $_clearField(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
