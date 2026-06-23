// This is a generated file - do not edit.
//
// Generated from client/vault.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $0;

import '../shared/vault.pbenum.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

enum Request_Payload { queryState, notSet }

class Request extends $pb.GeneratedMessage {
  factory Request({
    $0.Empty? queryState,
  }) {
    final result = create();
    if (queryState != null) result.queryState = queryState;
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
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.client.vault'),
      createEmptyInstance: create)
    ..oo(0, [1])
    ..aOM<$0.Empty>(1, _omitFieldNames ? '' : 'queryState',
        subBuilder: $0.Empty.create)
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
  Request_Payload whichPayload() => _Request_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
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
}

enum Response_Payload { state, notSet }

class Response extends $pb.GeneratedMessage {
  factory Response({
    $1.VaultState? state,
  }) {
    final result = create();
    if (state != null) result.state = state;
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
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.client.vault'),
      createEmptyInstance: create)
    ..oo(0, [1])
    ..aE<$1.VaultState>(1, _omitFieldNames ? '' : 'state',
        enumValues: $1.VaultState.values)
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
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $1.VaultState get state => $_getN(0);
  @$pb.TagNumber(1)
  set state($1.VaultState value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasState() => $_has(0);
  @$pb.TagNumber(1)
  void clearState() => $_clearField(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
