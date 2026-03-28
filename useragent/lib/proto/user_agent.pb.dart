// This is a generated file - do not edit.
//
// Generated from user_agent.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $1;

import 'client.pb.dart' as $0;
import 'evm.pb.dart' as $2;
import 'user_agent.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'user_agent.pbenum.dart';

class SdkClientRevokeRequest extends $pb.GeneratedMessage {
  factory SdkClientRevokeRequest({
    $core.int? clientId,
  }) {
    final result = create();
    if (clientId != null) result.clientId = clientId;
    return result;
  }

  SdkClientRevokeRequest._();

  factory SdkClientRevokeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientRevokeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientRevokeRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'clientId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientRevokeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientRevokeRequest copyWith(
          void Function(SdkClientRevokeRequest) updates) =>
      super.copyWith((message) => updates(message as SdkClientRevokeRequest))
          as SdkClientRevokeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientRevokeRequest create() => SdkClientRevokeRequest._();
  @$core.override
  SdkClientRevokeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientRevokeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientRevokeRequest>(create);
  static SdkClientRevokeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get clientId => $_getIZ(0);
  @$pb.TagNumber(1)
  set clientId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);
}

class SdkClientEntry extends $pb.GeneratedMessage {
  factory SdkClientEntry({
    $core.int? id,
    $core.List<$core.int>? pubkey,
    $0.ClientInfo? info,
    $core.int? createdAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (pubkey != null) result.pubkey = pubkey;
    if (info != null) result.info = info;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  SdkClientEntry._();

  factory SdkClientEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientEntry',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..aOM<$0.ClientInfo>(3, _omitFieldNames ? '' : 'info',
        subBuilder: $0.ClientInfo.create)
    ..aI(4, _omitFieldNames ? '' : 'createdAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientEntry copyWith(void Function(SdkClientEntry) updates) =>
      super.copyWith((message) => updates(message as SdkClientEntry))
          as SdkClientEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientEntry create() => SdkClientEntry._();
  @$core.override
  SdkClientEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientEntry>(create);
  static SdkClientEntry? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get pubkey => $_getN(1);
  @$pb.TagNumber(2)
  set pubkey($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPubkey() => $_has(1);
  @$pb.TagNumber(2)
  void clearPubkey() => $_clearField(2);

  @$pb.TagNumber(3)
  $0.ClientInfo get info => $_getN(2);
  @$pb.TagNumber(3)
  set info($0.ClientInfo value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasInfo() => $_has(2);
  @$pb.TagNumber(3)
  void clearInfo() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.ClientInfo ensureInfo() => $_ensure(2);

  @$pb.TagNumber(4)
  $core.int get createdAt => $_getIZ(3);
  @$pb.TagNumber(4)
  set createdAt($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedAt() => $_clearField(4);
}

class SdkClientList extends $pb.GeneratedMessage {
  factory SdkClientList({
    $core.Iterable<SdkClientEntry>? clients,
  }) {
    final result = create();
    if (clients != null) result.clients.addAll(clients);
    return result;
  }

  SdkClientList._();

  factory SdkClientList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientList',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..pPM<SdkClientEntry>(1, _omitFieldNames ? '' : 'clients',
        subBuilder: SdkClientEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientList copyWith(void Function(SdkClientList) updates) =>
      super.copyWith((message) => updates(message as SdkClientList))
          as SdkClientList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientList create() => SdkClientList._();
  @$core.override
  SdkClientList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientList>(create);
  static SdkClientList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SdkClientEntry> get clients => $_getList(0);
}

enum SdkClientRevokeResponse_Result { ok, error, notSet }

class SdkClientRevokeResponse extends $pb.GeneratedMessage {
  factory SdkClientRevokeResponse({
    $1.Empty? ok,
    SdkClientError? error,
  }) {
    final result = create();
    if (ok != null) result.ok = ok;
    if (error != null) result.error = error;
    return result;
  }

  SdkClientRevokeResponse._();

  factory SdkClientRevokeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientRevokeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SdkClientRevokeResponse_Result>
      _SdkClientRevokeResponse_ResultByTag = {
    1: SdkClientRevokeResponse_Result.ok,
    2: SdkClientRevokeResponse_Result.error,
    0: SdkClientRevokeResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientRevokeResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<$1.Empty>(1, _omitFieldNames ? '' : 'ok', subBuilder: $1.Empty.create)
    ..aE<SdkClientError>(2, _omitFieldNames ? '' : 'error',
        enumValues: SdkClientError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientRevokeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientRevokeResponse copyWith(
          void Function(SdkClientRevokeResponse) updates) =>
      super.copyWith((message) => updates(message as SdkClientRevokeResponse))
          as SdkClientRevokeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientRevokeResponse create() => SdkClientRevokeResponse._();
  @$core.override
  SdkClientRevokeResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientRevokeResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientRevokeResponse>(create);
  static SdkClientRevokeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  SdkClientRevokeResponse_Result whichResult() =>
      _SdkClientRevokeResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $1.Empty get ok => $_getN(0);
  @$pb.TagNumber(1)
  set ok($1.Empty value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasOk() => $_has(0);
  @$pb.TagNumber(1)
  void clearOk() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Empty ensureOk() => $_ensure(0);

  @$pb.TagNumber(2)
  SdkClientError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(SdkClientError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

enum SdkClientListResponse_Result { clients, error, notSet }

class SdkClientListResponse extends $pb.GeneratedMessage {
  factory SdkClientListResponse({
    SdkClientList? clients,
    SdkClientError? error,
  }) {
    final result = create();
    if (clients != null) result.clients = clients;
    if (error != null) result.error = error;
    return result;
  }

  SdkClientListResponse._();

  factory SdkClientListResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientListResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SdkClientListResponse_Result>
      _SdkClientListResponse_ResultByTag = {
    1: SdkClientListResponse_Result.clients,
    2: SdkClientListResponse_Result.error,
    0: SdkClientListResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientListResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<SdkClientList>(1, _omitFieldNames ? '' : 'clients',
        subBuilder: SdkClientList.create)
    ..aE<SdkClientError>(2, _omitFieldNames ? '' : 'error',
        enumValues: SdkClientError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientListResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientListResponse copyWith(
          void Function(SdkClientListResponse) updates) =>
      super.copyWith((message) => updates(message as SdkClientListResponse))
          as SdkClientListResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientListResponse create() => SdkClientListResponse._();
  @$core.override
  SdkClientListResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientListResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientListResponse>(create);
  static SdkClientListResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  SdkClientListResponse_Result whichResult() =>
      _SdkClientListResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  SdkClientList get clients => $_getN(0);
  @$pb.TagNumber(1)
  set clients(SdkClientList value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasClients() => $_has(0);
  @$pb.TagNumber(1)
  void clearClients() => $_clearField(1);
  @$pb.TagNumber(1)
  SdkClientList ensureClients() => $_ensure(0);

  @$pb.TagNumber(2)
  SdkClientError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(SdkClientError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

class AuthChallengeRequest extends $pb.GeneratedMessage {
  factory AuthChallengeRequest({
    $core.List<$core.int>? pubkey,
    $core.String? bootstrapToken,
    KeyType? keyType,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    if (bootstrapToken != null) result.bootstrapToken = bootstrapToken;
    if (keyType != null) result.keyType = keyType;
    return result;
  }

  AuthChallengeRequest._();

  factory AuthChallengeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuthChallengeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuthChallengeRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..aOS(2, _omitFieldNames ? '' : 'bootstrapToken')
    ..aE<KeyType>(3, _omitFieldNames ? '' : 'keyType',
        enumValues: KeyType.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthChallengeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthChallengeRequest copyWith(void Function(AuthChallengeRequest) updates) =>
      super.copyWith((message) => updates(message as AuthChallengeRequest))
          as AuthChallengeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuthChallengeRequest create() => AuthChallengeRequest._();
  @$core.override
  AuthChallengeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuthChallengeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AuthChallengeRequest>(create);
  static AuthChallengeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get pubkey => $_getN(0);
  @$pb.TagNumber(1)
  set pubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearPubkey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get bootstrapToken => $_getSZ(1);
  @$pb.TagNumber(2)
  set bootstrapToken($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBootstrapToken() => $_has(1);
  @$pb.TagNumber(2)
  void clearBootstrapToken() => $_clearField(2);

  @$pb.TagNumber(3)
  KeyType get keyType => $_getN(2);
  @$pb.TagNumber(3)
  set keyType(KeyType value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasKeyType() => $_has(2);
  @$pb.TagNumber(3)
  void clearKeyType() => $_clearField(3);
}

class AuthChallenge extends $pb.GeneratedMessage {
  factory AuthChallenge({
    $core.int? nonce,
  }) {
    final result = create();
    if (nonce != null) result.nonce = nonce;
    return result;
  }

  AuthChallenge._();

  factory AuthChallenge.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuthChallenge.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuthChallenge',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aI(2, _omitFieldNames ? '' : 'nonce')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthChallenge clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthChallenge copyWith(void Function(AuthChallenge) updates) =>
      super.copyWith((message) => updates(message as AuthChallenge))
          as AuthChallenge;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuthChallenge create() => AuthChallenge._();
  @$core.override
  AuthChallenge createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuthChallenge getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AuthChallenge>(create);
  static AuthChallenge? _defaultInstance;

  @$pb.TagNumber(2)
  $core.int get nonce => $_getIZ(0);
  @$pb.TagNumber(2)
  set nonce($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(2)
  $core.bool hasNonce() => $_has(0);
  @$pb.TagNumber(2)
  void clearNonce() => $_clearField(2);
}

class AuthChallengeSolution extends $pb.GeneratedMessage {
  factory AuthChallengeSolution({
    $core.List<$core.int>? signature,
  }) {
    final result = create();
    if (signature != null) result.signature = signature;
    return result;
  }

  AuthChallengeSolution._();

  factory AuthChallengeSolution.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuthChallengeSolution.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuthChallengeSolution',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'signature', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthChallengeSolution clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthChallengeSolution copyWith(
          void Function(AuthChallengeSolution) updates) =>
      super.copyWith((message) => updates(message as AuthChallengeSolution))
          as AuthChallengeSolution;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuthChallengeSolution create() => AuthChallengeSolution._();
  @$core.override
  AuthChallengeSolution createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuthChallengeSolution getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AuthChallengeSolution>(create);
  static AuthChallengeSolution? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get signature => $_getN(0);
  @$pb.TagNumber(1)
  set signature($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSignature() => $_has(0);
  @$pb.TagNumber(1)
  void clearSignature() => $_clearField(1);
}

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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
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

class BootstrapEncryptedKey extends $pb.GeneratedMessage {
  factory BootstrapEncryptedKey({
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

  BootstrapEncryptedKey._();

  factory BootstrapEncryptedKey.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory BootstrapEncryptedKey.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BootstrapEncryptedKey',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'nonce', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'ciphertext', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'associatedData', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BootstrapEncryptedKey clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BootstrapEncryptedKey copyWith(
          void Function(BootstrapEncryptedKey) updates) =>
      super.copyWith((message) => updates(message as BootstrapEncryptedKey))
          as BootstrapEncryptedKey;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static BootstrapEncryptedKey create() => BootstrapEncryptedKey._();
  @$core.override
  BootstrapEncryptedKey createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static BootstrapEncryptedKey getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BootstrapEncryptedKey>(create);
  static BootstrapEncryptedKey? _defaultInstance;

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

class SdkClientConnectionRequest extends $pb.GeneratedMessage {
  factory SdkClientConnectionRequest({
    $core.List<$core.int>? pubkey,
    $0.ClientInfo? info,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    if (info != null) result.info = info;
    return result;
  }

  SdkClientConnectionRequest._();

  factory SdkClientConnectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientConnectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientConnectionRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..aOM<$0.ClientInfo>(2, _omitFieldNames ? '' : 'info',
        subBuilder: $0.ClientInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientConnectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientConnectionRequest copyWith(
          void Function(SdkClientConnectionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as SdkClientConnectionRequest))
          as SdkClientConnectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientConnectionRequest create() => SdkClientConnectionRequest._();
  @$core.override
  SdkClientConnectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientConnectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientConnectionRequest>(create);
  static SdkClientConnectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get pubkey => $_getN(0);
  @$pb.TagNumber(1)
  set pubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearPubkey() => $_clearField(1);

  @$pb.TagNumber(2)
  $0.ClientInfo get info => $_getN(1);
  @$pb.TagNumber(2)
  set info($0.ClientInfo value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasInfo() => $_has(1);
  @$pb.TagNumber(2)
  void clearInfo() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.ClientInfo ensureInfo() => $_ensure(1);
}

class SdkClientConnectionResponse extends $pb.GeneratedMessage {
  factory SdkClientConnectionResponse({
    $core.bool? approved,
    $core.List<$core.int>? pubkey,
  }) {
    final result = create();
    if (approved != null) result.approved = approved;
    if (pubkey != null) result.pubkey = pubkey;
    return result;
  }

  SdkClientConnectionResponse._();

  factory SdkClientConnectionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientConnectionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientConnectionResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'approved')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientConnectionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientConnectionResponse copyWith(
          void Function(SdkClientConnectionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as SdkClientConnectionResponse))
          as SdkClientConnectionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientConnectionResponse create() =>
      SdkClientConnectionResponse._();
  @$core.override
  SdkClientConnectionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientConnectionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientConnectionResponse>(create);
  static SdkClientConnectionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get approved => $_getBF(0);
  @$pb.TagNumber(1)
  set approved($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasApproved() => $_has(0);
  @$pb.TagNumber(1)
  void clearApproved() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get pubkey => $_getN(1);
  @$pb.TagNumber(2)
  set pubkey($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPubkey() => $_has(1);
  @$pb.TagNumber(2)
  void clearPubkey() => $_clearField(2);
}

class SdkClientConnectionCancel extends $pb.GeneratedMessage {
  factory SdkClientConnectionCancel({
    $core.List<$core.int>? pubkey,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    return result;
  }

  SdkClientConnectionCancel._();

  factory SdkClientConnectionCancel.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientConnectionCancel.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientConnectionCancel',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientConnectionCancel clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientConnectionCancel copyWith(
          void Function(SdkClientConnectionCancel) updates) =>
      super.copyWith((message) => updates(message as SdkClientConnectionCancel))
          as SdkClientConnectionCancel;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientConnectionCancel create() => SdkClientConnectionCancel._();
  @$core.override
  SdkClientConnectionCancel createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientConnectionCancel getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientConnectionCancel>(create);
  static SdkClientConnectionCancel? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get pubkey => $_getN(0);
  @$pb.TagNumber(1)
  set pubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearPubkey() => $_clearField(1);
}

class WalletAccess extends $pb.GeneratedMessage {
  factory WalletAccess({
    $core.int? walletId,
    $core.int? sdkClientId,
  }) {
    final result = create();
    if (walletId != null) result.walletId = walletId;
    if (sdkClientId != null) result.sdkClientId = sdkClientId;
    return result;
  }

  WalletAccess._();

  factory WalletAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WalletAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WalletAccess',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'walletId')
    ..aI(2, _omitFieldNames ? '' : 'sdkClientId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletAccess copyWith(void Function(WalletAccess) updates) =>
      super.copyWith((message) => updates(message as WalletAccess))
          as WalletAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WalletAccess create() => WalletAccess._();
  @$core.override
  WalletAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WalletAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WalletAccess>(create);
  static WalletAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get walletId => $_getIZ(0);
  @$pb.TagNumber(1)
  set walletId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get sdkClientId => $_getIZ(1);
  @$pb.TagNumber(2)
  set sdkClientId($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSdkClientId() => $_has(1);
  @$pb.TagNumber(2)
  void clearSdkClientId() => $_clearField(2);
}

class SdkClientWalletAccess extends $pb.GeneratedMessage {
  factory SdkClientWalletAccess({
    $core.int? id,
    WalletAccess? access,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (access != null) result.access = access;
    return result;
  }

  SdkClientWalletAccess._();

  factory SdkClientWalletAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientWalletAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientWalletAccess',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id')
    ..aOM<WalletAccess>(2, _omitFieldNames ? '' : 'access',
        subBuilder: WalletAccess.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientWalletAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientWalletAccess copyWith(
          void Function(SdkClientWalletAccess) updates) =>
      super.copyWith((message) => updates(message as SdkClientWalletAccess))
          as SdkClientWalletAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientWalletAccess create() => SdkClientWalletAccess._();
  @$core.override
  SdkClientWalletAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientWalletAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientWalletAccess>(create);
  static SdkClientWalletAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  WalletAccess get access => $_getN(1);
  @$pb.TagNumber(2)
  set access(WalletAccess value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAccess() => $_has(1);
  @$pb.TagNumber(2)
  void clearAccess() => $_clearField(2);
  @$pb.TagNumber(2)
  WalletAccess ensureAccess() => $_ensure(1);
}

class SdkClientGrantWalletAccess extends $pb.GeneratedMessage {
  factory SdkClientGrantWalletAccess({
    $core.Iterable<WalletAccess>? accesses,
  }) {
    final result = create();
    if (accesses != null) result.accesses.addAll(accesses);
    return result;
  }

  SdkClientGrantWalletAccess._();

  factory SdkClientGrantWalletAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientGrantWalletAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientGrantWalletAccess',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..pPM<WalletAccess>(1, _omitFieldNames ? '' : 'accesses',
        subBuilder: WalletAccess.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientGrantWalletAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientGrantWalletAccess copyWith(
          void Function(SdkClientGrantWalletAccess) updates) =>
      super.copyWith(
              (message) => updates(message as SdkClientGrantWalletAccess))
          as SdkClientGrantWalletAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientGrantWalletAccess create() => SdkClientGrantWalletAccess._();
  @$core.override
  SdkClientGrantWalletAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientGrantWalletAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientGrantWalletAccess>(create);
  static SdkClientGrantWalletAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WalletAccess> get accesses => $_getList(0);
}

class SdkClientRevokeWalletAccess extends $pb.GeneratedMessage {
  factory SdkClientRevokeWalletAccess({
    $core.Iterable<$core.int>? accesses,
  }) {
    final result = create();
    if (accesses != null) result.accesses.addAll(accesses);
    return result;
  }

  SdkClientRevokeWalletAccess._();

  factory SdkClientRevokeWalletAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SdkClientRevokeWalletAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SdkClientRevokeWalletAccess',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..p<$core.int>(1, _omitFieldNames ? '' : 'accesses', $pb.PbFieldType.K3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientRevokeWalletAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SdkClientRevokeWalletAccess copyWith(
          void Function(SdkClientRevokeWalletAccess) updates) =>
      super.copyWith(
              (message) => updates(message as SdkClientRevokeWalletAccess))
          as SdkClientRevokeWalletAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SdkClientRevokeWalletAccess create() =>
      SdkClientRevokeWalletAccess._();
  @$core.override
  SdkClientRevokeWalletAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SdkClientRevokeWalletAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SdkClientRevokeWalletAccess>(create);
  static SdkClientRevokeWalletAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.int> get accesses => $_getList(0);
}

class ListWalletAccessResponse extends $pb.GeneratedMessage {
  factory ListWalletAccessResponse({
    $core.Iterable<SdkClientWalletAccess>? accesses,
  }) {
    final result = create();
    if (accesses != null) result.accesses.addAll(accesses);
    return result;
  }

  ListWalletAccessResponse._();

  factory ListWalletAccessResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListWalletAccessResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListWalletAccessResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..pPM<SdkClientWalletAccess>(1, _omitFieldNames ? '' : 'accesses',
        subBuilder: SdkClientWalletAccess.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListWalletAccessResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListWalletAccessResponse copyWith(
          void Function(ListWalletAccessResponse) updates) =>
      super.copyWith((message) => updates(message as ListWalletAccessResponse))
          as ListWalletAccessResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListWalletAccessResponse create() => ListWalletAccessResponse._();
  @$core.override
  ListWalletAccessResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListWalletAccessResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListWalletAccessResponse>(create);
  static ListWalletAccessResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SdkClientWalletAccess> get accesses => $_getList(0);
}

enum UserAgentRequest_Payload {
  authChallengeRequest,
  authChallengeSolution,
  unsealStart,
  unsealEncryptedKey,
  queryVaultState,
  evmWalletCreate,
  evmWalletList,
  evmGrantCreate,
  evmGrantDelete,
  evmGrantList,
  sdkClientConnectionResponse,
  sdkClientRevoke,
  sdkClientList,
  bootstrapEncryptedKey,
  grantWalletAccess,
  revokeWalletAccess,
  listWalletAccess,
  notSet
}

class UserAgentRequest extends $pb.GeneratedMessage {
  factory UserAgentRequest({
    AuthChallengeRequest? authChallengeRequest,
    AuthChallengeSolution? authChallengeSolution,
    UnsealStart? unsealStart,
    UnsealEncryptedKey? unsealEncryptedKey,
    $1.Empty? queryVaultState,
    $1.Empty? evmWalletCreate,
    $1.Empty? evmWalletList,
    $2.EvmGrantCreateRequest? evmGrantCreate,
    $2.EvmGrantDeleteRequest? evmGrantDelete,
    $2.EvmGrantListRequest? evmGrantList,
    SdkClientConnectionResponse? sdkClientConnectionResponse,
    SdkClientRevokeRequest? sdkClientRevoke,
    $1.Empty? sdkClientList,
    BootstrapEncryptedKey? bootstrapEncryptedKey,
    SdkClientGrantWalletAccess? grantWalletAccess,
    $core.int? id,
    SdkClientRevokeWalletAccess? revokeWalletAccess,
    $1.Empty? listWalletAccess,
  }) {
    final result = create();
    if (authChallengeRequest != null)
      result.authChallengeRequest = authChallengeRequest;
    if (authChallengeSolution != null)
      result.authChallengeSolution = authChallengeSolution;
    if (unsealStart != null) result.unsealStart = unsealStart;
    if (unsealEncryptedKey != null)
      result.unsealEncryptedKey = unsealEncryptedKey;
    if (queryVaultState != null) result.queryVaultState = queryVaultState;
    if (evmWalletCreate != null) result.evmWalletCreate = evmWalletCreate;
    if (evmWalletList != null) result.evmWalletList = evmWalletList;
    if (evmGrantCreate != null) result.evmGrantCreate = evmGrantCreate;
    if (evmGrantDelete != null) result.evmGrantDelete = evmGrantDelete;
    if (evmGrantList != null) result.evmGrantList = evmGrantList;
    if (sdkClientConnectionResponse != null)
      result.sdkClientConnectionResponse = sdkClientConnectionResponse;
    if (sdkClientRevoke != null) result.sdkClientRevoke = sdkClientRevoke;
    if (sdkClientList != null) result.sdkClientList = sdkClientList;
    if (bootstrapEncryptedKey != null)
      result.bootstrapEncryptedKey = bootstrapEncryptedKey;
    if (grantWalletAccess != null) result.grantWalletAccess = grantWalletAccess;
    if (id != null) result.id = id;
    if (revokeWalletAccess != null)
      result.revokeWalletAccess = revokeWalletAccess;
    if (listWalletAccess != null) result.listWalletAccess = listWalletAccess;
    return result;
  }

  UserAgentRequest._();

  factory UserAgentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UserAgentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, UserAgentRequest_Payload>
      _UserAgentRequest_PayloadByTag = {
    1: UserAgentRequest_Payload.authChallengeRequest,
    2: UserAgentRequest_Payload.authChallengeSolution,
    3: UserAgentRequest_Payload.unsealStart,
    4: UserAgentRequest_Payload.unsealEncryptedKey,
    5: UserAgentRequest_Payload.queryVaultState,
    6: UserAgentRequest_Payload.evmWalletCreate,
    7: UserAgentRequest_Payload.evmWalletList,
    8: UserAgentRequest_Payload.evmGrantCreate,
    9: UserAgentRequest_Payload.evmGrantDelete,
    10: UserAgentRequest_Payload.evmGrantList,
    11: UserAgentRequest_Payload.sdkClientConnectionResponse,
    12: UserAgentRequest_Payload.sdkClientRevoke,
    13: UserAgentRequest_Payload.sdkClientList,
    14: UserAgentRequest_Payload.bootstrapEncryptedKey,
    15: UserAgentRequest_Payload.grantWalletAccess,
    17: UserAgentRequest_Payload.revokeWalletAccess,
    18: UserAgentRequest_Payload.listWalletAccess,
    0: UserAgentRequest_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UserAgentRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18])
    ..aOM<AuthChallengeRequest>(
        1, _omitFieldNames ? '' : 'authChallengeRequest',
        subBuilder: AuthChallengeRequest.create)
    ..aOM<AuthChallengeSolution>(
        2, _omitFieldNames ? '' : 'authChallengeSolution',
        subBuilder: AuthChallengeSolution.create)
    ..aOM<UnsealStart>(3, _omitFieldNames ? '' : 'unsealStart',
        subBuilder: UnsealStart.create)
    ..aOM<UnsealEncryptedKey>(4, _omitFieldNames ? '' : 'unsealEncryptedKey',
        subBuilder: UnsealEncryptedKey.create)
    ..aOM<$1.Empty>(5, _omitFieldNames ? '' : 'queryVaultState',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(6, _omitFieldNames ? '' : 'evmWalletCreate',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(7, _omitFieldNames ? '' : 'evmWalletList',
        subBuilder: $1.Empty.create)
    ..aOM<$2.EvmGrantCreateRequest>(8, _omitFieldNames ? '' : 'evmGrantCreate',
        subBuilder: $2.EvmGrantCreateRequest.create)
    ..aOM<$2.EvmGrantDeleteRequest>(9, _omitFieldNames ? '' : 'evmGrantDelete',
        subBuilder: $2.EvmGrantDeleteRequest.create)
    ..aOM<$2.EvmGrantListRequest>(10, _omitFieldNames ? '' : 'evmGrantList',
        subBuilder: $2.EvmGrantListRequest.create)
    ..aOM<SdkClientConnectionResponse>(
        11, _omitFieldNames ? '' : 'sdkClientConnectionResponse',
        subBuilder: SdkClientConnectionResponse.create)
    ..aOM<SdkClientRevokeRequest>(12, _omitFieldNames ? '' : 'sdkClientRevoke',
        subBuilder: SdkClientRevokeRequest.create)
    ..aOM<$1.Empty>(13, _omitFieldNames ? '' : 'sdkClientList',
        subBuilder: $1.Empty.create)
    ..aOM<BootstrapEncryptedKey>(
        14, _omitFieldNames ? '' : 'bootstrapEncryptedKey',
        subBuilder: BootstrapEncryptedKey.create)
    ..aOM<SdkClientGrantWalletAccess>(
        15, _omitFieldNames ? '' : 'grantWalletAccess',
        subBuilder: SdkClientGrantWalletAccess.create)
    ..aI(16, _omitFieldNames ? '' : 'id')
    ..aOM<SdkClientRevokeWalletAccess>(
        17, _omitFieldNames ? '' : 'revokeWalletAccess',
        subBuilder: SdkClientRevokeWalletAccess.create)
    ..aOM<$1.Empty>(18, _omitFieldNames ? '' : 'listWalletAccess',
        subBuilder: $1.Empty.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UserAgentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UserAgentRequest copyWith(void Function(UserAgentRequest) updates) =>
      super.copyWith((message) => updates(message as UserAgentRequest))
          as UserAgentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UserAgentRequest create() => UserAgentRequest._();
  @$core.override
  UserAgentRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UserAgentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UserAgentRequest>(create);
  static UserAgentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  UserAgentRequest_Payload whichPayload() =>
      _UserAgentRequest_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(17)
  @$pb.TagNumber(18)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  AuthChallengeRequest get authChallengeRequest => $_getN(0);
  @$pb.TagNumber(1)
  set authChallengeRequest(AuthChallengeRequest value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAuthChallengeRequest() => $_has(0);
  @$pb.TagNumber(1)
  void clearAuthChallengeRequest() => $_clearField(1);
  @$pb.TagNumber(1)
  AuthChallengeRequest ensureAuthChallengeRequest() => $_ensure(0);

  @$pb.TagNumber(2)
  AuthChallengeSolution get authChallengeSolution => $_getN(1);
  @$pb.TagNumber(2)
  set authChallengeSolution(AuthChallengeSolution value) =>
      $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAuthChallengeSolution() => $_has(1);
  @$pb.TagNumber(2)
  void clearAuthChallengeSolution() => $_clearField(2);
  @$pb.TagNumber(2)
  AuthChallengeSolution ensureAuthChallengeSolution() => $_ensure(1);

  @$pb.TagNumber(3)
  UnsealStart get unsealStart => $_getN(2);
  @$pb.TagNumber(3)
  set unsealStart(UnsealStart value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasUnsealStart() => $_has(2);
  @$pb.TagNumber(3)
  void clearUnsealStart() => $_clearField(3);
  @$pb.TagNumber(3)
  UnsealStart ensureUnsealStart() => $_ensure(2);

  @$pb.TagNumber(4)
  UnsealEncryptedKey get unsealEncryptedKey => $_getN(3);
  @$pb.TagNumber(4)
  set unsealEncryptedKey(UnsealEncryptedKey value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasUnsealEncryptedKey() => $_has(3);
  @$pb.TagNumber(4)
  void clearUnsealEncryptedKey() => $_clearField(4);
  @$pb.TagNumber(4)
  UnsealEncryptedKey ensureUnsealEncryptedKey() => $_ensure(3);

  @$pb.TagNumber(5)
  $1.Empty get queryVaultState => $_getN(4);
  @$pb.TagNumber(5)
  set queryVaultState($1.Empty value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasQueryVaultState() => $_has(4);
  @$pb.TagNumber(5)
  void clearQueryVaultState() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Empty ensureQueryVaultState() => $_ensure(4);

  @$pb.TagNumber(6)
  $1.Empty get evmWalletCreate => $_getN(5);
  @$pb.TagNumber(6)
  set evmWalletCreate($1.Empty value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasEvmWalletCreate() => $_has(5);
  @$pb.TagNumber(6)
  void clearEvmWalletCreate() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Empty ensureEvmWalletCreate() => $_ensure(5);

  @$pb.TagNumber(7)
  $1.Empty get evmWalletList => $_getN(6);
  @$pb.TagNumber(7)
  set evmWalletList($1.Empty value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEvmWalletList() => $_has(6);
  @$pb.TagNumber(7)
  void clearEvmWalletList() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Empty ensureEvmWalletList() => $_ensure(6);

  @$pb.TagNumber(8)
  $2.EvmGrantCreateRequest get evmGrantCreate => $_getN(7);
  @$pb.TagNumber(8)
  set evmGrantCreate($2.EvmGrantCreateRequest value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEvmGrantCreate() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvmGrantCreate() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.EvmGrantCreateRequest ensureEvmGrantCreate() => $_ensure(7);

  @$pb.TagNumber(9)
  $2.EvmGrantDeleteRequest get evmGrantDelete => $_getN(8);
  @$pb.TagNumber(9)
  set evmGrantDelete($2.EvmGrantDeleteRequest value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasEvmGrantDelete() => $_has(8);
  @$pb.TagNumber(9)
  void clearEvmGrantDelete() => $_clearField(9);
  @$pb.TagNumber(9)
  $2.EvmGrantDeleteRequest ensureEvmGrantDelete() => $_ensure(8);

  @$pb.TagNumber(10)
  $2.EvmGrantListRequest get evmGrantList => $_getN(9);
  @$pb.TagNumber(10)
  set evmGrantList($2.EvmGrantListRequest value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasEvmGrantList() => $_has(9);
  @$pb.TagNumber(10)
  void clearEvmGrantList() => $_clearField(10);
  @$pb.TagNumber(10)
  $2.EvmGrantListRequest ensureEvmGrantList() => $_ensure(9);

  @$pb.TagNumber(11)
  SdkClientConnectionResponse get sdkClientConnectionResponse => $_getN(10);
  @$pb.TagNumber(11)
  set sdkClientConnectionResponse(SdkClientConnectionResponse value) =>
      $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasSdkClientConnectionResponse() => $_has(10);
  @$pb.TagNumber(11)
  void clearSdkClientConnectionResponse() => $_clearField(11);
  @$pb.TagNumber(11)
  SdkClientConnectionResponse ensureSdkClientConnectionResponse() =>
      $_ensure(10);

  @$pb.TagNumber(12)
  SdkClientRevokeRequest get sdkClientRevoke => $_getN(11);
  @$pb.TagNumber(12)
  set sdkClientRevoke(SdkClientRevokeRequest value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasSdkClientRevoke() => $_has(11);
  @$pb.TagNumber(12)
  void clearSdkClientRevoke() => $_clearField(12);
  @$pb.TagNumber(12)
  SdkClientRevokeRequest ensureSdkClientRevoke() => $_ensure(11);

  @$pb.TagNumber(13)
  $1.Empty get sdkClientList => $_getN(12);
  @$pb.TagNumber(13)
  set sdkClientList($1.Empty value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasSdkClientList() => $_has(12);
  @$pb.TagNumber(13)
  void clearSdkClientList() => $_clearField(13);
  @$pb.TagNumber(13)
  $1.Empty ensureSdkClientList() => $_ensure(12);

  @$pb.TagNumber(14)
  BootstrapEncryptedKey get bootstrapEncryptedKey => $_getN(13);
  @$pb.TagNumber(14)
  set bootstrapEncryptedKey(BootstrapEncryptedKey value) =>
      $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasBootstrapEncryptedKey() => $_has(13);
  @$pb.TagNumber(14)
  void clearBootstrapEncryptedKey() => $_clearField(14);
  @$pb.TagNumber(14)
  BootstrapEncryptedKey ensureBootstrapEncryptedKey() => $_ensure(13);

  @$pb.TagNumber(15)
  SdkClientGrantWalletAccess get grantWalletAccess => $_getN(14);
  @$pb.TagNumber(15)
  set grantWalletAccess(SdkClientGrantWalletAccess value) =>
      $_setField(15, value);
  @$pb.TagNumber(15)
  $core.bool hasGrantWalletAccess() => $_has(14);
  @$pb.TagNumber(15)
  void clearGrantWalletAccess() => $_clearField(15);
  @$pb.TagNumber(15)
  SdkClientGrantWalletAccess ensureGrantWalletAccess() => $_ensure(14);

  @$pb.TagNumber(16)
  $core.int get id => $_getIZ(15);
  @$pb.TagNumber(16)
  set id($core.int value) => $_setSignedInt32(15, value);
  @$pb.TagNumber(16)
  $core.bool hasId() => $_has(15);
  @$pb.TagNumber(16)
  void clearId() => $_clearField(16);

  @$pb.TagNumber(17)
  SdkClientRevokeWalletAccess get revokeWalletAccess => $_getN(16);
  @$pb.TagNumber(17)
  set revokeWalletAccess(SdkClientRevokeWalletAccess value) =>
      $_setField(17, value);
  @$pb.TagNumber(17)
  $core.bool hasRevokeWalletAccess() => $_has(16);
  @$pb.TagNumber(17)
  void clearRevokeWalletAccess() => $_clearField(17);
  @$pb.TagNumber(17)
  SdkClientRevokeWalletAccess ensureRevokeWalletAccess() => $_ensure(16);

  @$pb.TagNumber(18)
  $1.Empty get listWalletAccess => $_getN(17);
  @$pb.TagNumber(18)
  set listWalletAccess($1.Empty value) => $_setField(18, value);
  @$pb.TagNumber(18)
  $core.bool hasListWalletAccess() => $_has(17);
  @$pb.TagNumber(18)
  void clearListWalletAccess() => $_clearField(18);
  @$pb.TagNumber(18)
  $1.Empty ensureListWalletAccess() => $_ensure(17);
}

enum UserAgentResponse_Payload {
  authChallenge,
  authResult,
  unsealStartResponse,
  unsealResult,
  vaultState,
  evmWalletCreate,
  evmWalletList,
  evmGrantCreate,
  evmGrantDelete,
  evmGrantList,
  sdkClientConnectionRequest,
  sdkClientConnectionCancel,
  sdkClientRevokeResponse,
  sdkClientListResponse,
  bootstrapResult,
  listWalletAccessResponse,
  notSet
}

class UserAgentResponse extends $pb.GeneratedMessage {
  factory UserAgentResponse({
    AuthChallenge? authChallenge,
    AuthResult? authResult,
    UnsealStartResponse? unsealStartResponse,
    UnsealResult? unsealResult,
    VaultState? vaultState,
    $2.WalletCreateResponse? evmWalletCreate,
    $2.WalletListResponse? evmWalletList,
    $2.EvmGrantCreateResponse? evmGrantCreate,
    $2.EvmGrantDeleteResponse? evmGrantDelete,
    $2.EvmGrantListResponse? evmGrantList,
    SdkClientConnectionRequest? sdkClientConnectionRequest,
    SdkClientConnectionCancel? sdkClientConnectionCancel,
    SdkClientRevokeResponse? sdkClientRevokeResponse,
    SdkClientListResponse? sdkClientListResponse,
    BootstrapResult? bootstrapResult,
    $core.int? id,
    ListWalletAccessResponse? listWalletAccessResponse,
  }) {
    final result = create();
    if (authChallenge != null) result.authChallenge = authChallenge;
    if (authResult != null) result.authResult = authResult;
    if (unsealStartResponse != null)
      result.unsealStartResponse = unsealStartResponse;
    if (unsealResult != null) result.unsealResult = unsealResult;
    if (vaultState != null) result.vaultState = vaultState;
    if (evmWalletCreate != null) result.evmWalletCreate = evmWalletCreate;
    if (evmWalletList != null) result.evmWalletList = evmWalletList;
    if (evmGrantCreate != null) result.evmGrantCreate = evmGrantCreate;
    if (evmGrantDelete != null) result.evmGrantDelete = evmGrantDelete;
    if (evmGrantList != null) result.evmGrantList = evmGrantList;
    if (sdkClientConnectionRequest != null)
      result.sdkClientConnectionRequest = sdkClientConnectionRequest;
    if (sdkClientConnectionCancel != null)
      result.sdkClientConnectionCancel = sdkClientConnectionCancel;
    if (sdkClientRevokeResponse != null)
      result.sdkClientRevokeResponse = sdkClientRevokeResponse;
    if (sdkClientListResponse != null)
      result.sdkClientListResponse = sdkClientListResponse;
    if (bootstrapResult != null) result.bootstrapResult = bootstrapResult;
    if (id != null) result.id = id;
    if (listWalletAccessResponse != null)
      result.listWalletAccessResponse = listWalletAccessResponse;
    return result;
  }

  UserAgentResponse._();

  factory UserAgentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UserAgentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, UserAgentResponse_Payload>
      _UserAgentResponse_PayloadByTag = {
    1: UserAgentResponse_Payload.authChallenge,
    2: UserAgentResponse_Payload.authResult,
    3: UserAgentResponse_Payload.unsealStartResponse,
    4: UserAgentResponse_Payload.unsealResult,
    5: UserAgentResponse_Payload.vaultState,
    6: UserAgentResponse_Payload.evmWalletCreate,
    7: UserAgentResponse_Payload.evmWalletList,
    8: UserAgentResponse_Payload.evmGrantCreate,
    9: UserAgentResponse_Payload.evmGrantDelete,
    10: UserAgentResponse_Payload.evmGrantList,
    11: UserAgentResponse_Payload.sdkClientConnectionRequest,
    12: UserAgentResponse_Payload.sdkClientConnectionCancel,
    13: UserAgentResponse_Payload.sdkClientRevokeResponse,
    14: UserAgentResponse_Payload.sdkClientListResponse,
    15: UserAgentResponse_Payload.bootstrapResult,
    17: UserAgentResponse_Payload.listWalletAccessResponse,
    0: UserAgentResponse_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UserAgentResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17])
    ..aOM<AuthChallenge>(1, _omitFieldNames ? '' : 'authChallenge',
        subBuilder: AuthChallenge.create)
    ..aE<AuthResult>(2, _omitFieldNames ? '' : 'authResult',
        enumValues: AuthResult.values)
    ..aOM<UnsealStartResponse>(3, _omitFieldNames ? '' : 'unsealStartResponse',
        subBuilder: UnsealStartResponse.create)
    ..aE<UnsealResult>(4, _omitFieldNames ? '' : 'unsealResult',
        enumValues: UnsealResult.values)
    ..aE<VaultState>(5, _omitFieldNames ? '' : 'vaultState',
        enumValues: VaultState.values)
    ..aOM<$2.WalletCreateResponse>(6, _omitFieldNames ? '' : 'evmWalletCreate',
        subBuilder: $2.WalletCreateResponse.create)
    ..aOM<$2.WalletListResponse>(7, _omitFieldNames ? '' : 'evmWalletList',
        subBuilder: $2.WalletListResponse.create)
    ..aOM<$2.EvmGrantCreateResponse>(8, _omitFieldNames ? '' : 'evmGrantCreate',
        subBuilder: $2.EvmGrantCreateResponse.create)
    ..aOM<$2.EvmGrantDeleteResponse>(9, _omitFieldNames ? '' : 'evmGrantDelete',
        subBuilder: $2.EvmGrantDeleteResponse.create)
    ..aOM<$2.EvmGrantListResponse>(10, _omitFieldNames ? '' : 'evmGrantList',
        subBuilder: $2.EvmGrantListResponse.create)
    ..aOM<SdkClientConnectionRequest>(
        11, _omitFieldNames ? '' : 'sdkClientConnectionRequest',
        subBuilder: SdkClientConnectionRequest.create)
    ..aOM<SdkClientConnectionCancel>(
        12, _omitFieldNames ? '' : 'sdkClientConnectionCancel',
        subBuilder: SdkClientConnectionCancel.create)
    ..aOM<SdkClientRevokeResponse>(
        13, _omitFieldNames ? '' : 'sdkClientRevokeResponse',
        subBuilder: SdkClientRevokeResponse.create)
    ..aOM<SdkClientListResponse>(
        14, _omitFieldNames ? '' : 'sdkClientListResponse',
        subBuilder: SdkClientListResponse.create)
    ..aE<BootstrapResult>(15, _omitFieldNames ? '' : 'bootstrapResult',
        enumValues: BootstrapResult.values)
    ..aI(16, _omitFieldNames ? '' : 'id')
    ..aOM<ListWalletAccessResponse>(
        17, _omitFieldNames ? '' : 'listWalletAccessResponse',
        subBuilder: ListWalletAccessResponse.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UserAgentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UserAgentResponse copyWith(void Function(UserAgentResponse) updates) =>
      super.copyWith((message) => updates(message as UserAgentResponse))
          as UserAgentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UserAgentResponse create() => UserAgentResponse._();
  @$core.override
  UserAgentResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UserAgentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UserAgentResponse>(create);
  static UserAgentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(17)
  UserAgentResponse_Payload whichPayload() =>
      _UserAgentResponse_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  @$pb.TagNumber(13)
  @$pb.TagNumber(14)
  @$pb.TagNumber(15)
  @$pb.TagNumber(17)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  AuthChallenge get authChallenge => $_getN(0);
  @$pb.TagNumber(1)
  set authChallenge(AuthChallenge value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAuthChallenge() => $_has(0);
  @$pb.TagNumber(1)
  void clearAuthChallenge() => $_clearField(1);
  @$pb.TagNumber(1)
  AuthChallenge ensureAuthChallenge() => $_ensure(0);

  @$pb.TagNumber(2)
  AuthResult get authResult => $_getN(1);
  @$pb.TagNumber(2)
  set authResult(AuthResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAuthResult() => $_has(1);
  @$pb.TagNumber(2)
  void clearAuthResult() => $_clearField(2);

  @$pb.TagNumber(3)
  UnsealStartResponse get unsealStartResponse => $_getN(2);
  @$pb.TagNumber(3)
  set unsealStartResponse(UnsealStartResponse value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasUnsealStartResponse() => $_has(2);
  @$pb.TagNumber(3)
  void clearUnsealStartResponse() => $_clearField(3);
  @$pb.TagNumber(3)
  UnsealStartResponse ensureUnsealStartResponse() => $_ensure(2);

  @$pb.TagNumber(4)
  UnsealResult get unsealResult => $_getN(3);
  @$pb.TagNumber(4)
  set unsealResult(UnsealResult value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasUnsealResult() => $_has(3);
  @$pb.TagNumber(4)
  void clearUnsealResult() => $_clearField(4);

  @$pb.TagNumber(5)
  VaultState get vaultState => $_getN(4);
  @$pb.TagNumber(5)
  set vaultState(VaultState value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasVaultState() => $_has(4);
  @$pb.TagNumber(5)
  void clearVaultState() => $_clearField(5);

  @$pb.TagNumber(6)
  $2.WalletCreateResponse get evmWalletCreate => $_getN(5);
  @$pb.TagNumber(6)
  set evmWalletCreate($2.WalletCreateResponse value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasEvmWalletCreate() => $_has(5);
  @$pb.TagNumber(6)
  void clearEvmWalletCreate() => $_clearField(6);
  @$pb.TagNumber(6)
  $2.WalletCreateResponse ensureEvmWalletCreate() => $_ensure(5);

  @$pb.TagNumber(7)
  $2.WalletListResponse get evmWalletList => $_getN(6);
  @$pb.TagNumber(7)
  set evmWalletList($2.WalletListResponse value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEvmWalletList() => $_has(6);
  @$pb.TagNumber(7)
  void clearEvmWalletList() => $_clearField(7);
  @$pb.TagNumber(7)
  $2.WalletListResponse ensureEvmWalletList() => $_ensure(6);

  @$pb.TagNumber(8)
  $2.EvmGrantCreateResponse get evmGrantCreate => $_getN(7);
  @$pb.TagNumber(8)
  set evmGrantCreate($2.EvmGrantCreateResponse value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEvmGrantCreate() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvmGrantCreate() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.EvmGrantCreateResponse ensureEvmGrantCreate() => $_ensure(7);

  @$pb.TagNumber(9)
  $2.EvmGrantDeleteResponse get evmGrantDelete => $_getN(8);
  @$pb.TagNumber(9)
  set evmGrantDelete($2.EvmGrantDeleteResponse value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasEvmGrantDelete() => $_has(8);
  @$pb.TagNumber(9)
  void clearEvmGrantDelete() => $_clearField(9);
  @$pb.TagNumber(9)
  $2.EvmGrantDeleteResponse ensureEvmGrantDelete() => $_ensure(8);

  @$pb.TagNumber(10)
  $2.EvmGrantListResponse get evmGrantList => $_getN(9);
  @$pb.TagNumber(10)
  set evmGrantList($2.EvmGrantListResponse value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasEvmGrantList() => $_has(9);
  @$pb.TagNumber(10)
  void clearEvmGrantList() => $_clearField(10);
  @$pb.TagNumber(10)
  $2.EvmGrantListResponse ensureEvmGrantList() => $_ensure(9);

  @$pb.TagNumber(11)
  SdkClientConnectionRequest get sdkClientConnectionRequest => $_getN(10);
  @$pb.TagNumber(11)
  set sdkClientConnectionRequest(SdkClientConnectionRequest value) =>
      $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasSdkClientConnectionRequest() => $_has(10);
  @$pb.TagNumber(11)
  void clearSdkClientConnectionRequest() => $_clearField(11);
  @$pb.TagNumber(11)
  SdkClientConnectionRequest ensureSdkClientConnectionRequest() => $_ensure(10);

  @$pb.TagNumber(12)
  SdkClientConnectionCancel get sdkClientConnectionCancel => $_getN(11);
  @$pb.TagNumber(12)
  set sdkClientConnectionCancel(SdkClientConnectionCancel value) =>
      $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasSdkClientConnectionCancel() => $_has(11);
  @$pb.TagNumber(12)
  void clearSdkClientConnectionCancel() => $_clearField(12);
  @$pb.TagNumber(12)
  SdkClientConnectionCancel ensureSdkClientConnectionCancel() => $_ensure(11);

  @$pb.TagNumber(13)
  SdkClientRevokeResponse get sdkClientRevokeResponse => $_getN(12);
  @$pb.TagNumber(13)
  set sdkClientRevokeResponse(SdkClientRevokeResponse value) =>
      $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasSdkClientRevokeResponse() => $_has(12);
  @$pb.TagNumber(13)
  void clearSdkClientRevokeResponse() => $_clearField(13);
  @$pb.TagNumber(13)
  SdkClientRevokeResponse ensureSdkClientRevokeResponse() => $_ensure(12);

  @$pb.TagNumber(14)
  SdkClientListResponse get sdkClientListResponse => $_getN(13);
  @$pb.TagNumber(14)
  set sdkClientListResponse(SdkClientListResponse value) =>
      $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasSdkClientListResponse() => $_has(13);
  @$pb.TagNumber(14)
  void clearSdkClientListResponse() => $_clearField(14);
  @$pb.TagNumber(14)
  SdkClientListResponse ensureSdkClientListResponse() => $_ensure(13);

  @$pb.TagNumber(15)
  BootstrapResult get bootstrapResult => $_getN(14);
  @$pb.TagNumber(15)
  set bootstrapResult(BootstrapResult value) => $_setField(15, value);
  @$pb.TagNumber(15)
  $core.bool hasBootstrapResult() => $_has(14);
  @$pb.TagNumber(15)
  void clearBootstrapResult() => $_clearField(15);

  @$pb.TagNumber(16)
  $core.int get id => $_getIZ(15);
  @$pb.TagNumber(16)
  set id($core.int value) => $_setSignedInt32(15, value);
  @$pb.TagNumber(16)
  $core.bool hasId() => $_has(15);
  @$pb.TagNumber(16)
  void clearId() => $_clearField(16);

  @$pb.TagNumber(17)
  ListWalletAccessResponse get listWalletAccessResponse => $_getN(16);
  @$pb.TagNumber(17)
  set listWalletAccessResponse(ListWalletAccessResponse value) =>
      $_setField(17, value);
  @$pb.TagNumber(17)
  $core.bool hasListWalletAccessResponse() => $_has(16);
  @$pb.TagNumber(17)
  void clearListWalletAccessResponse() => $_clearField(17);
  @$pb.TagNumber(17)
  ListWalletAccessResponse ensureListWalletAccessResponse() => $_ensure(16);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
