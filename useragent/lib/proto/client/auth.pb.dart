// This is a generated file - do not edit.
//
// Generated from client/auth.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import '../shared/client.pb.dart' as $0;
import 'auth.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'auth.pbenum.dart';

class AuthChallengeRequest extends $pb.GeneratedMessage {
  factory AuthChallengeRequest({
    $core.List<$core.int>? pubkey,
    $0.ClientInfo? clientInfo,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    if (clientInfo != null) result.clientInfo = clientInfo;
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
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.auth'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..aOM<$0.ClientInfo>(2, _omitFieldNames ? '' : 'clientInfo',
        subBuilder: $0.ClientInfo.create)
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
  $0.ClientInfo get clientInfo => $_getN(1);
  @$pb.TagNumber(2)
  set clientInfo($0.ClientInfo value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasClientInfo() => $_has(1);
  @$pb.TagNumber(2)
  void clearClientInfo() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.ClientInfo ensureClientInfo() => $_ensure(1);
}

class AuthChallenge extends $pb.GeneratedMessage {
  factory AuthChallenge({
    $fixnum.Int64? timestampNanos,
    $core.List<$core.int>? random,
  }) {
    final result = create();
    if (timestampNanos != null) result.timestampNanos = timestampNanos;
    if (random != null) result.random = random;
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
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.auth'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'timestampNanos', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'random', $pb.PbFieldType.OY)
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
  $fixnum.Int64 get timestampNanos => $_getI64(0);
  @$pb.TagNumber(1)
  set timestampNanos($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTimestampNanos() => $_has(0);
  @$pb.TagNumber(1)
  void clearTimestampNanos() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get random => $_getN(1);
  @$pb.TagNumber(2)
  set random($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRandom() => $_has(1);
  @$pb.TagNumber(2)
  void clearRandom() => $_clearField(2);
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
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.auth'),
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

enum Request_Payload { challengeRequest, challengeSolution, notSet }

class Request extends $pb.GeneratedMessage {
  factory Request({
    AuthChallengeRequest? challengeRequest,
    AuthChallengeSolution? challengeSolution,
  }) {
    final result = create();
    if (challengeRequest != null) result.challengeRequest = challengeRequest;
    if (challengeSolution != null) result.challengeSolution = challengeSolution;
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
    1: Request_Payload.challengeRequest,
    2: Request_Payload.challengeSolution,
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.auth'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<AuthChallengeRequest>(1, _omitFieldNames ? '' : 'challengeRequest',
        subBuilder: AuthChallengeRequest.create)
    ..aOM<AuthChallengeSolution>(2, _omitFieldNames ? '' : 'challengeSolution',
        subBuilder: AuthChallengeSolution.create)
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
  AuthChallengeRequest get challengeRequest => $_getN(0);
  @$pb.TagNumber(1)
  set challengeRequest(AuthChallengeRequest value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasChallengeRequest() => $_has(0);
  @$pb.TagNumber(1)
  void clearChallengeRequest() => $_clearField(1);
  @$pb.TagNumber(1)
  AuthChallengeRequest ensureChallengeRequest() => $_ensure(0);

  @$pb.TagNumber(2)
  AuthChallengeSolution get challengeSolution => $_getN(1);
  @$pb.TagNumber(2)
  set challengeSolution(AuthChallengeSolution value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasChallengeSolution() => $_has(1);
  @$pb.TagNumber(2)
  void clearChallengeSolution() => $_clearField(2);
  @$pb.TagNumber(2)
  AuthChallengeSolution ensureChallengeSolution() => $_ensure(1);
}

enum Response_Payload { challenge, result, notSet }

class Response extends $pb.GeneratedMessage {
  factory Response({
    AuthChallenge? challenge,
    AuthResult? result,
  }) {
    final result$ = create();
    if (challenge != null) result$.challenge = challenge;
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
    1: Response_Payload.challenge,
    2: Response_Payload.result,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client.auth'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<AuthChallenge>(1, _omitFieldNames ? '' : 'challenge',
        subBuilder: AuthChallenge.create)
    ..aE<AuthResult>(2, _omitFieldNames ? '' : 'result',
        enumValues: AuthResult.values)
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
  AuthChallenge get challenge => $_getN(0);
  @$pb.TagNumber(1)
  set challenge(AuthChallenge value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasChallenge() => $_has(0);
  @$pb.TagNumber(1)
  void clearChallenge() => $_clearField(1);
  @$pb.TagNumber(1)
  AuthChallenge ensureChallenge() => $_ensure(0);

  @$pb.TagNumber(2)
  AuthResult get result => $_getN(1);
  @$pb.TagNumber(2)
  set result(AuthResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasResult() => $_has(1);
  @$pb.TagNumber(2)
  void clearResult() => $_clearField(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
