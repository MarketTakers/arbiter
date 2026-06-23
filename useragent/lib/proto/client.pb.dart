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

import 'client/auth.pb.dart' as $0;
import 'client/evm.pb.dart' as $2;
import 'client/vault.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

enum ClientRequest_Payload { auth, vault, evm, notSet }

class ClientRequest extends $pb.GeneratedMessage {
  factory ClientRequest({
    $0.Request? auth,
    $1.Request? vault,
    $2.Request? evm,
    $core.int? requestId,
  }) {
    final result = create();
    if (auth != null) result.auth = auth;
    if (vault != null) result.vault = vault;
    if (evm != null) result.evm = evm;
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
    1: ClientRequest_Payload.auth,
    2: ClientRequest_Payload.vault,
    3: ClientRequest_Payload.evm,
    0: ClientRequest_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aOM<$0.Request>(1, _omitFieldNames ? '' : 'auth',
        subBuilder: $0.Request.create)
    ..aOM<$1.Request>(2, _omitFieldNames ? '' : 'vault',
        subBuilder: $1.Request.create)
    ..aOM<$2.Request>(3, _omitFieldNames ? '' : 'evm',
        subBuilder: $2.Request.create)
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
  $core.int get requestId => $_getIZ(3);
  @$pb.TagNumber(4)
  set requestId($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRequestId() => $_has(3);
  @$pb.TagNumber(4)
  void clearRequestId() => $_clearField(4);
}

enum ClientResponse_Payload { auth, vault, evm, notSet }

class ClientResponse extends $pb.GeneratedMessage {
  factory ClientResponse({
    $0.Response? auth,
    $1.Response? vault,
    $2.Response? evm,
    $core.int? requestId,
  }) {
    final result = create();
    if (auth != null) result.auth = auth;
    if (vault != null) result.vault = vault;
    if (evm != null) result.evm = evm;
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
    1: ClientResponse_Payload.auth,
    2: ClientResponse_Payload.vault,
    3: ClientResponse_Payload.evm,
    0: ClientResponse_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aOM<$0.Response>(1, _omitFieldNames ? '' : 'auth',
        subBuilder: $0.Response.create)
    ..aOM<$1.Response>(2, _omitFieldNames ? '' : 'vault',
        subBuilder: $1.Response.create)
    ..aOM<$2.Response>(3, _omitFieldNames ? '' : 'evm',
        subBuilder: $2.Response.create)
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
  ClientResponse_Payload whichPayload() =>
      _ClientResponse_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
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

  @$pb.TagNumber(7)
  $core.int get requestId => $_getIZ(3);
  @$pb.TagNumber(7)
  set requestId($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(7)
  $core.bool hasRequestId() => $_has(3);
  @$pb.TagNumber(7)
  void clearRequestId() => $_clearField(7);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
