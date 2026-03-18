// This is a generated file - do not edit.
//
// Generated from client.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $0;

import 'client.pbenum.dart';
import 'evm.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'client.pbenum.dart';

class AuthChallengeRequest extends $pb.GeneratedMessage {
  factory AuthChallengeRequest({
    $core.List<$core.int>? pubkey,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
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
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
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
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
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
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
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

enum ClientRequest_Payload {
  authChallengeRequest,
  authChallengeSolution,
  queryVaultState,
  notSet
}

class ClientRequest extends $pb.GeneratedMessage {
  factory ClientRequest({
    AuthChallengeRequest? authChallengeRequest,
    AuthChallengeSolution? authChallengeSolution,
    $0.Empty? queryVaultState,
    $core.int? requestId,
  }) {
    final result = create();
    if (authChallengeRequest != null)
      result.authChallengeRequest = authChallengeRequest;
    if (authChallengeSolution != null)
      result.authChallengeSolution = authChallengeSolution;
    if (queryVaultState != null) result.queryVaultState = queryVaultState;
    if (requestId != null) result.requestId = requestId;
    return result;
  }

  ClientRequest._();

  factory ClientRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ClientRequest_Payload>
      _ClientRequest_PayloadByTag = {
    1: ClientRequest_Payload.authChallengeRequest,
    2: ClientRequest_Payload.authChallengeSolution,
    3: ClientRequest_Payload.queryVaultState,
    0: ClientRequest_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aOM<AuthChallengeRequest>(
        1, _omitFieldNames ? '' : 'authChallengeRequest',
        subBuilder: AuthChallengeRequest.create)
    ..aOM<AuthChallengeSolution>(
        2, _omitFieldNames ? '' : 'authChallengeSolution',
        subBuilder: AuthChallengeSolution.create)
    ..aOM<$0.Empty>(3, _omitFieldNames ? '' : 'queryVaultState',
        subBuilder: $0.Empty.create)
    ..aI(4, _omitFieldNames ? '' : 'requestId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientRequest copyWith(void Function(ClientRequest) updates) =>
      super.copyWith((message) => updates(message as ClientRequest))
          as ClientRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientRequest create() => ClientRequest._();
  @$core.override
  ClientRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientRequest>(create);
  static ClientRequest? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  ClientRequest_Payload whichPayload() =>
      _ClientRequest_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
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
  $0.Empty get queryVaultState => $_getN(2);
  @$pb.TagNumber(3)
  set queryVaultState($0.Empty value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasQueryVaultState() => $_has(2);
  @$pb.TagNumber(3)
  void clearQueryVaultState() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.Empty ensureQueryVaultState() => $_ensure(2);

  @$pb.TagNumber(4)
  $core.int get requestId => $_getIZ(3);
  @$pb.TagNumber(4)
  set requestId($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRequestId() => $_has(3);
  @$pb.TagNumber(4)
  void clearRequestId() => $_clearField(4);
}

enum ClientResponse_Payload {
  authChallenge,
  authResult,
  evmSignTransaction,
  evmAnalyzeTransaction,
  vaultState,
  notSet
}

class ClientResponse extends $pb.GeneratedMessage {
  factory ClientResponse({
    AuthChallenge? authChallenge,
    AuthResult? authResult,
    $1.EvmSignTransactionResponse? evmSignTransaction,
    $1.EvmAnalyzeTransactionResponse? evmAnalyzeTransaction,
    VaultState? vaultState,
    $core.int? requestId,
  }) {
    final result = create();
    if (authChallenge != null) result.authChallenge = authChallenge;
    if (authResult != null) result.authResult = authResult;
    if (evmSignTransaction != null)
      result.evmSignTransaction = evmSignTransaction;
    if (evmAnalyzeTransaction != null)
      result.evmAnalyzeTransaction = evmAnalyzeTransaction;
    if (vaultState != null) result.vaultState = vaultState;
    if (requestId != null) result.requestId = requestId;
    return result;
  }

  ClientResponse._();

  factory ClientResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ClientResponse_Payload>
      _ClientResponse_PayloadByTag = {
    1: ClientResponse_Payload.authChallenge,
    2: ClientResponse_Payload.authResult,
    3: ClientResponse_Payload.evmSignTransaction,
    4: ClientResponse_Payload.evmAnalyzeTransaction,
    6: ClientResponse_Payload.vaultState,
    0: ClientResponse_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 6])
    ..aOM<AuthChallenge>(1, _omitFieldNames ? '' : 'authChallenge',
        subBuilder: AuthChallenge.create)
    ..aE<AuthResult>(2, _omitFieldNames ? '' : 'authResult',
        enumValues: AuthResult.values)
    ..aOM<$1.EvmSignTransactionResponse>(
        3, _omitFieldNames ? '' : 'evmSignTransaction',
        subBuilder: $1.EvmSignTransactionResponse.create)
    ..aOM<$1.EvmAnalyzeTransactionResponse>(
        4, _omitFieldNames ? '' : 'evmAnalyzeTransaction',
        subBuilder: $1.EvmAnalyzeTransactionResponse.create)
    ..aE<VaultState>(6, _omitFieldNames ? '' : 'vaultState',
        enumValues: VaultState.values)
    ..aI(7, _omitFieldNames ? '' : 'requestId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientResponse copyWith(void Function(ClientResponse) updates) =>
      super.copyWith((message) => updates(message as ClientResponse))
          as ClientResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientResponse create() => ClientResponse._();
  @$core.override
  ClientResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientResponse>(create);
  static ClientResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(6)
  ClientResponse_Payload whichPayload() =>
      _ClientResponse_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(6)
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
  $1.EvmSignTransactionResponse get evmSignTransaction => $_getN(2);
  @$pb.TagNumber(3)
  set evmSignTransaction($1.EvmSignTransactionResponse value) =>
      $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasEvmSignTransaction() => $_has(2);
  @$pb.TagNumber(3)
  void clearEvmSignTransaction() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.EvmSignTransactionResponse ensureEvmSignTransaction() => $_ensure(2);

  @$pb.TagNumber(4)
  $1.EvmAnalyzeTransactionResponse get evmAnalyzeTransaction => $_getN(3);
  @$pb.TagNumber(4)
  set evmAnalyzeTransaction($1.EvmAnalyzeTransactionResponse value) =>
      $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasEvmAnalyzeTransaction() => $_has(3);
  @$pb.TagNumber(4)
  void clearEvmAnalyzeTransaction() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.EvmAnalyzeTransactionResponse ensureEvmAnalyzeTransaction() => $_ensure(3);

  @$pb.TagNumber(6)
  VaultState get vaultState => $_getN(4);
  @$pb.TagNumber(6)
  set vaultState(VaultState value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasVaultState() => $_has(4);
  @$pb.TagNumber(6)
  void clearVaultState() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get requestId => $_getIZ(5);
  @$pb.TagNumber(7)
  set requestId($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(7)
  $core.bool hasRequestId() => $_has(5);
  @$pb.TagNumber(7)
  void clearRequestId() => $_clearField(7);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
