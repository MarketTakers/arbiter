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

import 'client.pbenum.dart';
import 'evm.pb.dart' as $0;

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
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
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

enum ClientRequest_Payload {
  authChallengeRequest,
  authChallengeSolution,
  notSet
}

class ClientRequest extends $pb.GeneratedMessage {
  factory ClientRequest({
    AuthChallengeRequest? authChallengeRequest,
    AuthChallengeSolution? authChallengeSolution,
  }) {
    final result = create();
    if (authChallengeRequest != null)
      result.authChallengeRequest = authChallengeRequest;
    if (authChallengeSolution != null)
      result.authChallengeSolution = authChallengeSolution;
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
    0: ClientRequest_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<AuthChallengeRequest>(
        1, _omitFieldNames ? '' : 'authChallengeRequest',
        subBuilder: AuthChallengeRequest.create)
    ..aOM<AuthChallengeSolution>(
        2, _omitFieldNames ? '' : 'authChallengeSolution',
        subBuilder: AuthChallengeSolution.create)
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
  ClientRequest_Payload whichPayload() =>
      _ClientRequest_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
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
}

class ClientConnectError extends $pb.GeneratedMessage {
  factory ClientConnectError({
    ClientConnectError_Code? code,
  }) {
    final result = create();
    if (code != null) result.code = code;
    return result;
  }

  ClientConnectError._();

  factory ClientConnectError.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ClientConnectError.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientConnectError',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..aE<ClientConnectError_Code>(1, _omitFieldNames ? '' : 'code',
        enumValues: ClientConnectError_Code.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectError clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientConnectError copyWith(void Function(ClientConnectError) updates) =>
      super.copyWith((message) => updates(message as ClientConnectError))
          as ClientConnectError;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ClientConnectError create() => ClientConnectError._();
  @$core.override
  ClientConnectError createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ClientConnectError getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ClientConnectError>(create);
  static ClientConnectError? _defaultInstance;

  @$pb.TagNumber(1)
  ClientConnectError_Code get code => $_getN(0);
  @$pb.TagNumber(1)
  set code(ClientConnectError_Code value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCode() => $_has(0);
  @$pb.TagNumber(1)
  void clearCode() => $_clearField(1);
}

enum ClientResponse_Payload {
  authChallenge,
  authOk,
  evmSignTransaction,
  evmAnalyzeTransaction,
  clientConnectError,
  notSet
}

class ClientResponse extends $pb.GeneratedMessage {
  factory ClientResponse({
    AuthChallenge? authChallenge,
    AuthOk? authOk,
    $0.EvmSignTransactionResponse? evmSignTransaction,
    $0.EvmAnalyzeTransactionResponse? evmAnalyzeTransaction,
    ClientConnectError? clientConnectError,
  }) {
    final result = create();
    if (authChallenge != null) result.authChallenge = authChallenge;
    if (authOk != null) result.authOk = authOk;
    if (evmSignTransaction != null)
      result.evmSignTransaction = evmSignTransaction;
    if (evmAnalyzeTransaction != null)
      result.evmAnalyzeTransaction = evmAnalyzeTransaction;
    if (clientConnectError != null)
      result.clientConnectError = clientConnectError;
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
    2: ClientResponse_Payload.authOk,
    3: ClientResponse_Payload.evmSignTransaction,
    4: ClientResponse_Payload.evmAnalyzeTransaction,
    5: ClientResponse_Payload.clientConnectError,
    0: ClientResponse_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5])
    ..aOM<AuthChallenge>(1, _omitFieldNames ? '' : 'authChallenge',
        subBuilder: AuthChallenge.create)
    ..aOM<AuthOk>(2, _omitFieldNames ? '' : 'authOk', subBuilder: AuthOk.create)
    ..aOM<$0.EvmSignTransactionResponse>(
        3, _omitFieldNames ? '' : 'evmSignTransaction',
        subBuilder: $0.EvmSignTransactionResponse.create)
    ..aOM<$0.EvmAnalyzeTransactionResponse>(
        4, _omitFieldNames ? '' : 'evmAnalyzeTransaction',
        subBuilder: $0.EvmAnalyzeTransactionResponse.create)
    ..aOM<ClientConnectError>(5, _omitFieldNames ? '' : 'clientConnectError',
        subBuilder: ClientConnectError.create)
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
  @$pb.TagNumber(5)
  ClientResponse_Payload whichPayload() =>
      _ClientResponse_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
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
  $0.EvmSignTransactionResponse get evmSignTransaction => $_getN(2);
  @$pb.TagNumber(3)
  set evmSignTransaction($0.EvmSignTransactionResponse value) =>
      $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasEvmSignTransaction() => $_has(2);
  @$pb.TagNumber(3)
  void clearEvmSignTransaction() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.EvmSignTransactionResponse ensureEvmSignTransaction() => $_ensure(2);

  @$pb.TagNumber(4)
  $0.EvmAnalyzeTransactionResponse get evmAnalyzeTransaction => $_getN(3);
  @$pb.TagNumber(4)
  set evmAnalyzeTransaction($0.EvmAnalyzeTransactionResponse value) =>
      $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasEvmAnalyzeTransaction() => $_has(3);
  @$pb.TagNumber(4)
  void clearEvmAnalyzeTransaction() => $_clearField(4);
  @$pb.TagNumber(4)
  $0.EvmAnalyzeTransactionResponse ensureEvmAnalyzeTransaction() => $_ensure(3);

  @$pb.TagNumber(5)
  ClientConnectError get clientConnectError => $_getN(4);
  @$pb.TagNumber(5)
  set clientConnectError(ClientConnectError value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasClientConnectError() => $_has(4);
  @$pb.TagNumber(5)
  void clearClientConnectError() => $_clearField(5);
  @$pb.TagNumber(5)
  ClientConnectError ensureClientConnectError() => $_ensure(4);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
