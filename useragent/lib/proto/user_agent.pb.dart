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

import 'user_agent/auth.pb.dart' as $0;
import 'user_agent/evm.pb.dart' as $2;
import 'user_agent/sdk_client.pb.dart' as $3;
import 'user_agent/vault/vault.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

enum UserAgentRequest_Payload { auth, vault, evm, sdkClient, notSet }

class UserAgentRequest extends $pb.GeneratedMessage {
  factory UserAgentRequest({
    $0.Request? auth,
    $1.Request? vault,
    $2.Request? evm,
    $3.Request? sdkClient,
    $core.int? id,
  }) {
    final result = create();
    if (auth != null) result.auth = auth;
    if (vault != null) result.vault = vault;
    if (evm != null) result.evm = evm;
    if (sdkClient != null) result.sdkClient = sdkClient;
    if (id != null) result.id = id;
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
    1: UserAgentRequest_Payload.auth,
    2: UserAgentRequest_Payload.vault,
    3: UserAgentRequest_Payload.evm,
    4: UserAgentRequest_Payload.sdkClient,
    0: UserAgentRequest_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UserAgentRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOM<$0.Request>(1, _omitFieldNames ? '' : 'auth',
        subBuilder: $0.Request.create)
    ..aOM<$1.Request>(2, _omitFieldNames ? '' : 'vault',
        subBuilder: $1.Request.create)
    ..aOM<$2.Request>(3, _omitFieldNames ? '' : 'evm',
        subBuilder: $2.Request.create)
    ..aOM<$3.Request>(4, _omitFieldNames ? '' : 'sdkClient',
        subBuilder: $3.Request.create)
    ..aI(16, _omitFieldNames ? '' : 'id')
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
  UserAgentRequest_Payload whichPayload() =>
      _UserAgentRequest_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $0.Request get auth => $_getN(0);
  @$pb.TagNumber(1)
  set auth($0.Request value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAuth() => $_has(0);
  @$pb.TagNumber(1)
  void clearAuth() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.Request ensureAuth() => $_ensure(0);

  @$pb.TagNumber(2)
  $1.Request get vault => $_getN(1);
  @$pb.TagNumber(2)
  set vault($1.Request value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasVault() => $_has(1);
  @$pb.TagNumber(2)
  void clearVault() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Request ensureVault() => $_ensure(1);

  @$pb.TagNumber(3)
  $2.Request get evm => $_getN(2);
  @$pb.TagNumber(3)
  set evm($2.Request value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasEvm() => $_has(2);
  @$pb.TagNumber(3)
  void clearEvm() => $_clearField(3);
  @$pb.TagNumber(3)
  $2.Request ensureEvm() => $_ensure(2);

  @$pb.TagNumber(4)
  $3.Request get sdkClient => $_getN(3);
  @$pb.TagNumber(4)
  set sdkClient($3.Request value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSdkClient() => $_has(3);
  @$pb.TagNumber(4)
  void clearSdkClient() => $_clearField(4);
  @$pb.TagNumber(4)
  $3.Request ensureSdkClient() => $_ensure(3);

  @$pb.TagNumber(16)
  $core.int get id => $_getIZ(4);
  @$pb.TagNumber(16)
  set id($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(16)
  $core.bool hasId() => $_has(4);
  @$pb.TagNumber(16)
  void clearId() => $_clearField(16);
}

enum UserAgentResponse_Payload { auth, vault, evm, sdkClient, notSet }

class UserAgentResponse extends $pb.GeneratedMessage {
  factory UserAgentResponse({
    $0.Response? auth,
    $1.Response? vault,
    $2.Response? evm,
    $3.Response? sdkClient,
    $core.int? id,
  }) {
    final result = create();
    if (auth != null) result.auth = auth;
    if (vault != null) result.vault = vault;
    if (evm != null) result.evm = evm;
    if (sdkClient != null) result.sdkClient = sdkClient;
    if (id != null) result.id = id;
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
    1: UserAgentResponse_Payload.auth,
    2: UserAgentResponse_Payload.vault,
    3: UserAgentResponse_Payload.evm,
    4: UserAgentResponse_Payload.sdkClient,
    0: UserAgentResponse_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UserAgentResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.user_agent'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOM<$0.Response>(1, _omitFieldNames ? '' : 'auth',
        subBuilder: $0.Response.create)
    ..aOM<$1.Response>(2, _omitFieldNames ? '' : 'vault',
        subBuilder: $1.Response.create)
    ..aOM<$2.Response>(3, _omitFieldNames ? '' : 'evm',
        subBuilder: $2.Response.create)
    ..aOM<$3.Response>(4, _omitFieldNames ? '' : 'sdkClient',
        subBuilder: $3.Response.create)
    ..aI(16, _omitFieldNames ? '' : 'id')
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
  UserAgentResponse_Payload whichPayload() =>
      _UserAgentResponse_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $0.Response get auth => $_getN(0);
  @$pb.TagNumber(1)
  set auth($0.Response value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAuth() => $_has(0);
  @$pb.TagNumber(1)
  void clearAuth() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.Response ensureAuth() => $_ensure(0);

  @$pb.TagNumber(2)
  $1.Response get vault => $_getN(1);
  @$pb.TagNumber(2)
  set vault($1.Response value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasVault() => $_has(1);
  @$pb.TagNumber(2)
  void clearVault() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Response ensureVault() => $_ensure(1);

  @$pb.TagNumber(3)
  $2.Response get evm => $_getN(2);
  @$pb.TagNumber(3)
  set evm($2.Response value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasEvm() => $_has(2);
  @$pb.TagNumber(3)
  void clearEvm() => $_clearField(3);
  @$pb.TagNumber(3)
  $2.Response ensureEvm() => $_ensure(2);

  @$pb.TagNumber(4)
  $3.Response get sdkClient => $_getN(3);
  @$pb.TagNumber(4)
  set sdkClient($3.Response value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSdkClient() => $_has(3);
  @$pb.TagNumber(4)
  void clearSdkClient() => $_clearField(4);
  @$pb.TagNumber(4)
  $3.Response ensureSdkClient() => $_ensure(3);

  @$pb.TagNumber(16)
  $core.int get id => $_getIZ(4);
  @$pb.TagNumber(16)
  set id($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(16)
  $core.bool hasId() => $_has(4);
  @$pb.TagNumber(16)
  void clearId() => $_clearField(16);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
