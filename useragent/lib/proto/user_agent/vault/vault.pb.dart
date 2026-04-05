// This is a generated file - do not edit.
//
// Generated from user_agent/vault/vault.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $0;

import '../../shared/vault.pbenum.dart' as $3;
import 'bootstrap.pb.dart' as $2;
import 'unseal.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

enum Request_Payload { queryState, unseal, bootstrap, notSet }

class Request extends $pb.GeneratedMessage {
  factory Request({
    $0.Empty? queryState,
    $1.Request? unseal,
    $2.Request? bootstrap,
  }) {
    final result = create();
    if (queryState != null) result.queryState = queryState;
    if (unseal != null) result.unseal = unseal;
    if (bootstrap != null) result.bootstrap = bootstrap;
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
    1: Request_Payload.queryState,
    2: Request_Payload.unseal,
    3: Request_Payload.bootstrap,
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aOM<$0.Empty>(1, _omitFieldNames ? '' : 'queryState',
        subBuilder: $0.Empty.create)
    ..aOM<$1.Request>(2, _omitFieldNames ? '' : 'unseal',
        subBuilder: $1.Request.create)
    ..aOM<$2.Request>(3, _omitFieldNames ? '' : 'bootstrap',
        subBuilder: $2.Request.create)
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
  @$pb.TagNumber(3)
  Request_Payload whichPayload() => _Request_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $0.Empty get queryState => $_getN(0);
  @$pb.TagNumber(1)
  set queryState($0.Empty value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasQueryState() => $_has(0);
  @$pb.TagNumber(1)
  void clearQueryState() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.Empty ensureQueryState() => $_ensure(0);

  @$pb.TagNumber(2)
  $1.Request get unseal => $_getN(1);
  @$pb.TagNumber(2)
  set unseal($1.Request value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasUnseal() => $_has(1);
  @$pb.TagNumber(2)
  void clearUnseal() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Request ensureUnseal() => $_ensure(1);

  @$pb.TagNumber(3)
  $2.Request get bootstrap => $_getN(2);
  @$pb.TagNumber(3)
  set bootstrap($2.Request value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasBootstrap() => $_has(2);
  @$pb.TagNumber(3)
  void clearBootstrap() => $_clearField(3);
  @$pb.TagNumber(3)
  $2.Request ensureBootstrap() => $_ensure(2);
}

enum Response_Payload { state, unseal, bootstrap, notSet }

class Response extends $pb.GeneratedMessage {
  factory Response({
    $3.VaultState? state,
    $1.Response? unseal,
    $2.Response? bootstrap,
  }) {
    final result = create();
    if (state != null) result.state = state;
    if (unseal != null) result.unseal = unseal;
    if (bootstrap != null) result.bootstrap = bootstrap;
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
    1: Response_Payload.state,
    2: Response_Payload.unseal,
    3: Response_Payload.bootstrap,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.vault'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aE<$3.VaultState>(1, _omitFieldNames ? '' : 'state',
        enumValues: $3.VaultState.values)
    ..aOM<$1.Response>(2, _omitFieldNames ? '' : 'unseal',
        subBuilder: $1.Response.create)
    ..aOM<$2.Response>(3, _omitFieldNames ? '' : 'bootstrap',
        subBuilder: $2.Response.create)
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
  @$pb.TagNumber(3)
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $3.VaultState get state => $_getN(0);
  @$pb.TagNumber(1)
  set state($3.VaultState value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasState() => $_has(0);
  @$pb.TagNumber(1)
  void clearState() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Response get unseal => $_getN(1);
  @$pb.TagNumber(2)
  set unseal($1.Response value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasUnseal() => $_has(1);
  @$pb.TagNumber(2)
  void clearUnseal() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Response ensureUnseal() => $_ensure(1);

  @$pb.TagNumber(3)
  $2.Response get bootstrap => $_getN(2);
  @$pb.TagNumber(3)
  set bootstrap($2.Response value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasBootstrap() => $_has(2);
  @$pb.TagNumber(3)
  void clearBootstrap() => $_clearField(3);
  @$pb.TagNumber(3)
  $2.Response ensureBootstrap() => $_ensure(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
