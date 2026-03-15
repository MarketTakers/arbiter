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
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $0;

import 'evm.pb.dart' as $1;
import 'user_agent.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'user_agent.pbenum.dart';

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
    $core.List<$core.int>? pubkey,
    $core.int? nonce,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
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
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
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

  @$pb.TagNumber(1)
  $core.List<$core.int> get pubkey => $_getN(0);
  @$pb.TagNumber(1)
  set pubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearPubkey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get nonce => $_getIZ(1);
  @$pb.TagNumber(2)
  set nonce($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNonce() => $_has(1);
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

class AuthOk extends $pb.GeneratedMessage {
  factory AuthOk() => create();

  AuthOk._();

  factory AuthOk.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuthOk.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuthOk',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthOk clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuthOk copyWith(void Function(AuthOk) updates) =>
      super.copyWith((message) => updates(message as AuthOk)) as AuthOk;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuthOk create() => AuthOk._();
  @$core.override
  AuthOk createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuthOk getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AuthOk>(create);
  static AuthOk? _defaultInstance;
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

class ClientConnectionRequest extends $pb.GeneratedMessage {
  factory ClientConnectionRequest({
    $core.List<$core.int>? pubkey,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    return result;
  }

  ClientConnectionRequest._();

  factory ClientConnectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientConnectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientConnectionRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectionRequest copyWith(
          void Function(ClientConnectionRequest) updates) =>
      super.copyWith((message) => updates(message as ClientConnectionRequest))
          as ClientConnectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientConnectionRequest create() => ClientConnectionRequest._();
  @$core.override
  ClientConnectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientConnectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientConnectionRequest>(create);
  static ClientConnectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get pubkey => $_getN(0);
  @$pb.TagNumber(1)
  set pubkey($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPubkey() => $_has(0);
  @$pb.TagNumber(1)
  void clearPubkey() => $_clearField(1);
}

class ClientConnectionResponse extends $pb.GeneratedMessage {
  factory ClientConnectionResponse({
    $core.bool? approved,
  }) {
    final result = create();
    if (approved != null) result.approved = approved;
    return result;
  }

  ClientConnectionResponse._();

  factory ClientConnectionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientConnectionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientConnectionResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'approved')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectionResponse copyWith(
          void Function(ClientConnectionResponse) updates) =>
      super.copyWith((message) => updates(message as ClientConnectionResponse))
          as ClientConnectionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientConnectionResponse create() => ClientConnectionResponse._();
  @$core.override
  ClientConnectionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientConnectionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientConnectionResponse>(create);
  static ClientConnectionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get approved => $_getBF(0);
  @$pb.TagNumber(1)
  set approved($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasApproved() => $_has(0);
  @$pb.TagNumber(1)
  void clearApproved() => $_clearField(1);
}

class ClientConnectionCancel extends $pb.GeneratedMessage {
  factory ClientConnectionCancel() => create();

  ClientConnectionCancel._();

  factory ClientConnectionCancel.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientConnectionCancel.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientConnectionCancel',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectionCancel clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectionCancel copyWith(
          void Function(ClientConnectionCancel) updates) =>
      super.copyWith((message) => updates(message as ClientConnectionCancel))
          as ClientConnectionCancel;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientConnectionCancel create() => ClientConnectionCancel._();
  @$core.override
  ClientConnectionCancel createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientConnectionCancel getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientConnectionCancel>(create);
  static ClientConnectionCancel? _defaultInstance;
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
  clientConnectionResponse,
  notSet
}

class UserAgentRequest extends $pb.GeneratedMessage {
  factory UserAgentRequest({
    AuthChallengeRequest? authChallengeRequest,
    AuthChallengeSolution? authChallengeSolution,
    UnsealStart? unsealStart,
    UnsealEncryptedKey? unsealEncryptedKey,
    $0.Empty? queryVaultState,
    $0.Empty? evmWalletCreate,
    $0.Empty? evmWalletList,
    $1.EvmGrantCreateRequest? evmGrantCreate,
    $1.EvmGrantDeleteRequest? evmGrantDelete,
    $1.EvmGrantListRequest? evmGrantList,
    ClientConnectionResponse? clientConnectionResponse,
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
    if (clientConnectionResponse != null)
      result.clientConnectionResponse = clientConnectionResponse;
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
    11: UserAgentRequest_Payload.clientConnectionResponse,
    0: UserAgentRequest_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UserAgentRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
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
    ..aOM<$0.Empty>(5, _omitFieldNames ? '' : 'queryVaultState',
        subBuilder: $0.Empty.create)
    ..aOM<$0.Empty>(6, _omitFieldNames ? '' : 'evmWalletCreate',
        subBuilder: $0.Empty.create)
    ..aOM<$0.Empty>(7, _omitFieldNames ? '' : 'evmWalletList',
        subBuilder: $0.Empty.create)
    ..aOM<$1.EvmGrantCreateRequest>(8, _omitFieldNames ? '' : 'evmGrantCreate',
        subBuilder: $1.EvmGrantCreateRequest.create)
    ..aOM<$1.EvmGrantDeleteRequest>(9, _omitFieldNames ? '' : 'evmGrantDelete',
        subBuilder: $1.EvmGrantDeleteRequest.create)
    ..aOM<$1.EvmGrantListRequest>(10, _omitFieldNames ? '' : 'evmGrantList',
        subBuilder: $1.EvmGrantListRequest.create)
    ..aOM<ClientConnectionResponse>(
        11, _omitFieldNames ? '' : 'clientConnectionResponse',
        subBuilder: ClientConnectionResponse.create)
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
  $0.Empty get queryVaultState => $_getN(4);
  @$pb.TagNumber(5)
  set queryVaultState($0.Empty value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasQueryVaultState() => $_has(4);
  @$pb.TagNumber(5)
  void clearQueryVaultState() => $_clearField(5);
  @$pb.TagNumber(5)
  $0.Empty ensureQueryVaultState() => $_ensure(4);

  @$pb.TagNumber(6)
  $0.Empty get evmWalletCreate => $_getN(5);
  @$pb.TagNumber(6)
  set evmWalletCreate($0.Empty value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasEvmWalletCreate() => $_has(5);
  @$pb.TagNumber(6)
  void clearEvmWalletCreate() => $_clearField(6);
  @$pb.TagNumber(6)
  $0.Empty ensureEvmWalletCreate() => $_ensure(5);

  @$pb.TagNumber(7)
  $0.Empty get evmWalletList => $_getN(6);
  @$pb.TagNumber(7)
  set evmWalletList($0.Empty value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEvmWalletList() => $_has(6);
  @$pb.TagNumber(7)
  void clearEvmWalletList() => $_clearField(7);
  @$pb.TagNumber(7)
  $0.Empty ensureEvmWalletList() => $_ensure(6);

  @$pb.TagNumber(8)
  $1.EvmGrantCreateRequest get evmGrantCreate => $_getN(7);
  @$pb.TagNumber(8)
  set evmGrantCreate($1.EvmGrantCreateRequest value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEvmGrantCreate() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvmGrantCreate() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.EvmGrantCreateRequest ensureEvmGrantCreate() => $_ensure(7);

  @$pb.TagNumber(9)
  $1.EvmGrantDeleteRequest get evmGrantDelete => $_getN(8);
  @$pb.TagNumber(9)
  set evmGrantDelete($1.EvmGrantDeleteRequest value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasEvmGrantDelete() => $_has(8);
  @$pb.TagNumber(9)
  void clearEvmGrantDelete() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.EvmGrantDeleteRequest ensureEvmGrantDelete() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.EvmGrantListRequest get evmGrantList => $_getN(9);
  @$pb.TagNumber(10)
  set evmGrantList($1.EvmGrantListRequest value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasEvmGrantList() => $_has(9);
  @$pb.TagNumber(10)
  void clearEvmGrantList() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.EvmGrantListRequest ensureEvmGrantList() => $_ensure(9);

  @$pb.TagNumber(11)
  ClientConnectionResponse get clientConnectionResponse => $_getN(10);
  @$pb.TagNumber(11)
  set clientConnectionResponse(ClientConnectionResponse value) =>
      $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasClientConnectionResponse() => $_has(10);
  @$pb.TagNumber(11)
  void clearClientConnectionResponse() => $_clearField(11);
  @$pb.TagNumber(11)
  ClientConnectionResponse ensureClientConnectionResponse() => $_ensure(10);
}

enum UserAgentResponse_Payload {
  authChallenge,
  authOk,
  unsealStartResponse,
  unsealResult,
  vaultState,
  evmWalletCreate,
  evmWalletList,
  evmGrantCreate,
  evmGrantDelete,
  evmGrantList,
  clientConnectionRequest,
  clientConnectionCancel,
  notSet
}

class UserAgentResponse extends $pb.GeneratedMessage {
  factory UserAgentResponse({
    AuthChallenge? authChallenge,
    AuthOk? authOk,
    UnsealStartResponse? unsealStartResponse,
    UnsealResult? unsealResult,
    VaultState? vaultState,
    $1.WalletCreateResponse? evmWalletCreate,
    $1.WalletListResponse? evmWalletList,
    $1.EvmGrantCreateResponse? evmGrantCreate,
    $1.EvmGrantDeleteResponse? evmGrantDelete,
    $1.EvmGrantListResponse? evmGrantList,
    ClientConnectionRequest? clientConnectionRequest,
    ClientConnectionCancel? clientConnectionCancel,
  }) {
    final result = create();
    if (authChallenge != null) result.authChallenge = authChallenge;
    if (authOk != null) result.authOk = authOk;
    if (unsealStartResponse != null)
      result.unsealStartResponse = unsealStartResponse;
    if (unsealResult != null) result.unsealResult = unsealResult;
    if (vaultState != null) result.vaultState = vaultState;
    if (evmWalletCreate != null) result.evmWalletCreate = evmWalletCreate;
    if (evmWalletList != null) result.evmWalletList = evmWalletList;
    if (evmGrantCreate != null) result.evmGrantCreate = evmGrantCreate;
    if (evmGrantDelete != null) result.evmGrantDelete = evmGrantDelete;
    if (evmGrantList != null) result.evmGrantList = evmGrantList;
    if (clientConnectionRequest != null)
      result.clientConnectionRequest = clientConnectionRequest;
    if (clientConnectionCancel != null)
      result.clientConnectionCancel = clientConnectionCancel;
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
    2: UserAgentResponse_Payload.authOk,
    3: UserAgentResponse_Payload.unsealStartResponse,
    4: UserAgentResponse_Payload.unsealResult,
    5: UserAgentResponse_Payload.vaultState,
    6: UserAgentResponse_Payload.evmWalletCreate,
    7: UserAgentResponse_Payload.evmWalletList,
    8: UserAgentResponse_Payload.evmGrantCreate,
    9: UserAgentResponse_Payload.evmGrantDelete,
    10: UserAgentResponse_Payload.evmGrantList,
    11: UserAgentResponse_Payload.clientConnectionRequest,
    12: UserAgentResponse_Payload.clientConnectionCancel,
    0: UserAgentResponse_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UserAgentResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
    ..aOM<AuthChallenge>(1, _omitFieldNames ? '' : 'authChallenge',
        subBuilder: AuthChallenge.create)
    ..aOM<AuthOk>(2, _omitFieldNames ? '' : 'authOk', subBuilder: AuthOk.create)
    ..aOM<UnsealStartResponse>(3, _omitFieldNames ? '' : 'unsealStartResponse',
        subBuilder: UnsealStartResponse.create)
    ..aE<UnsealResult>(4, _omitFieldNames ? '' : 'unsealResult',
        enumValues: UnsealResult.values)
    ..aE<VaultState>(5, _omitFieldNames ? '' : 'vaultState',
        enumValues: VaultState.values)
    ..aOM<$1.WalletCreateResponse>(6, _omitFieldNames ? '' : 'evmWalletCreate',
        subBuilder: $1.WalletCreateResponse.create)
    ..aOM<$1.WalletListResponse>(7, _omitFieldNames ? '' : 'evmWalletList',
        subBuilder: $1.WalletListResponse.create)
    ..aOM<$1.EvmGrantCreateResponse>(8, _omitFieldNames ? '' : 'evmGrantCreate',
        subBuilder: $1.EvmGrantCreateResponse.create)
    ..aOM<$1.EvmGrantDeleteResponse>(9, _omitFieldNames ? '' : 'evmGrantDelete',
        subBuilder: $1.EvmGrantDeleteResponse.create)
    ..aOM<$1.EvmGrantListResponse>(10, _omitFieldNames ? '' : 'evmGrantList',
        subBuilder: $1.EvmGrantListResponse.create)
    ..aOM<ClientConnectionRequest>(
        11, _omitFieldNames ? '' : 'clientConnectionRequest',
        subBuilder: ClientConnectionRequest.create)
    ..aOM<ClientConnectionCancel>(
        12, _omitFieldNames ? '' : 'clientConnectionCancel',
        subBuilder: ClientConnectionCancel.create)
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
  AuthOk get authOk => $_getN(1);
  @$pb.TagNumber(2)
  set authOk(AuthOk value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAuthOk() => $_has(1);
  @$pb.TagNumber(2)
  void clearAuthOk() => $_clearField(2);
  @$pb.TagNumber(2)
  AuthOk ensureAuthOk() => $_ensure(1);

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
  $1.WalletCreateResponse get evmWalletCreate => $_getN(5);
  @$pb.TagNumber(6)
  set evmWalletCreate($1.WalletCreateResponse value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasEvmWalletCreate() => $_has(5);
  @$pb.TagNumber(6)
  void clearEvmWalletCreate() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.WalletCreateResponse ensureEvmWalletCreate() => $_ensure(5);

  @$pb.TagNumber(7)
  $1.WalletListResponse get evmWalletList => $_getN(6);
  @$pb.TagNumber(7)
  set evmWalletList($1.WalletListResponse value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEvmWalletList() => $_has(6);
  @$pb.TagNumber(7)
  void clearEvmWalletList() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.WalletListResponse ensureEvmWalletList() => $_ensure(6);

  @$pb.TagNumber(8)
  $1.EvmGrantCreateResponse get evmGrantCreate => $_getN(7);
  @$pb.TagNumber(8)
  set evmGrantCreate($1.EvmGrantCreateResponse value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEvmGrantCreate() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvmGrantCreate() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.EvmGrantCreateResponse ensureEvmGrantCreate() => $_ensure(7);

  @$pb.TagNumber(9)
  $1.EvmGrantDeleteResponse get evmGrantDelete => $_getN(8);
  @$pb.TagNumber(9)
  set evmGrantDelete($1.EvmGrantDeleteResponse value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasEvmGrantDelete() => $_has(8);
  @$pb.TagNumber(9)
  void clearEvmGrantDelete() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.EvmGrantDeleteResponse ensureEvmGrantDelete() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.EvmGrantListResponse get evmGrantList => $_getN(9);
  @$pb.TagNumber(10)
  set evmGrantList($1.EvmGrantListResponse value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasEvmGrantList() => $_has(9);
  @$pb.TagNumber(10)
  void clearEvmGrantList() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.EvmGrantListResponse ensureEvmGrantList() => $_ensure(9);

  @$pb.TagNumber(11)
  ClientConnectionRequest get clientConnectionRequest => $_getN(10);
  @$pb.TagNumber(11)
  set clientConnectionRequest(ClientConnectionRequest value) =>
      $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasClientConnectionRequest() => $_has(10);
  @$pb.TagNumber(11)
  void clearClientConnectionRequest() => $_clearField(11);
  @$pb.TagNumber(11)
  ClientConnectionRequest ensureClientConnectionRequest() => $_ensure(10);

  @$pb.TagNumber(12)
  ClientConnectionCancel get clientConnectionCancel => $_getN(11);
  @$pb.TagNumber(12)
  set clientConnectionCancel(ClientConnectionCancel value) =>
      $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasClientConnectionCancel() => $_has(11);
  @$pb.TagNumber(12)
  void clearClientConnectionCancel() => $_clearField(12);
  @$pb.TagNumber(12)
  ClientConnectionCancel ensureClientConnectionCancel() => $_ensure(11);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
