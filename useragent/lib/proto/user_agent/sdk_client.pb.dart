// This is a generated file - do not edit.
//
// Generated from user_agent/sdk_client.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $1;

import '../shared/client.pb.dart' as $0;
import 'sdk_client.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'sdk_client.pbenum.dart';

class RevokeRequest extends $pb.GeneratedMessage {
  factory RevokeRequest({
    $core.int? clientId,
  }) {
    final result = create();
    if (clientId != null) result.clientId = clientId;
    return result;
  }

  RevokeRequest._();

  factory RevokeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RevokeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RevokeRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'clientId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeRequest copyWith(void Function(RevokeRequest) updates) =>
      super.copyWith((message) => updates(message as RevokeRequest))
          as RevokeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RevokeRequest create() => RevokeRequest._();
  @$core.override
  RevokeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RevokeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RevokeRequest>(create);
  static RevokeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get clientId => $_getIZ(0);
  @$pb.TagNumber(1)
  set clientId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);
}

class Entry extends $pb.GeneratedMessage {
  factory Entry({
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

  Entry._();

  factory Entry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Entry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Entry',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..aOM<$0.ClientInfo>(3, _omitFieldNames ? '' : 'info',
        subBuilder: $0.ClientInfo.create)
    ..aI(4, _omitFieldNames ? '' : 'createdAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Entry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Entry copyWith(void Function(Entry) updates) =>
      super.copyWith((message) => updates(message as Entry)) as Entry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Entry create() => Entry._();
  @$core.override
  Entry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Entry getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Entry>(create);
  static Entry? _defaultInstance;

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

class List_ extends $pb.GeneratedMessage {
  factory List_({
    $core.Iterable<Entry>? clients,
  }) {
    final result = create();
    if (clients != null) result.clients.addAll(clients);
    return result;
  }

  List_._();

  factory List_.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory List_.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'List',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..pPM<Entry>(1, _omitFieldNames ? '' : 'clients', subBuilder: Entry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  List_ clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  List_ copyWith(void Function(List_) updates) =>
      super.copyWith((message) => updates(message as List_)) as List_;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static List_ create() => List_._();
  @$core.override
  List_ createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static List_ getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<List_>(create);
  static List_? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Entry> get clients => $_getList(0);
}

enum RevokeResponse_Result { ok, error, notSet }

class RevokeResponse extends $pb.GeneratedMessage {
  factory RevokeResponse({
    $1.Empty? ok,
    Error? error,
  }) {
    final result = create();
    if (ok != null) result.ok = ok;
    if (error != null) result.error = error;
    return result;
  }

  RevokeResponse._();

  factory RevokeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RevokeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, RevokeResponse_Result>
      _RevokeResponse_ResultByTag = {
    1: RevokeResponse_Result.ok,
    2: RevokeResponse_Result.error,
    0: RevokeResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RevokeResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<$1.Empty>(1, _omitFieldNames ? '' : 'ok', subBuilder: $1.Empty.create)
    ..aE<Error>(2, _omitFieldNames ? '' : 'error', enumValues: Error.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeResponse copyWith(void Function(RevokeResponse) updates) =>
      super.copyWith((message) => updates(message as RevokeResponse))
          as RevokeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RevokeResponse create() => RevokeResponse._();
  @$core.override
  RevokeResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RevokeResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RevokeResponse>(create);
  static RevokeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  RevokeResponse_Result whichResult() =>
      _RevokeResponse_ResultByTag[$_whichOneof(0)]!;
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
  Error get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(Error value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

enum ListResponse_Result { clients, error, notSet }

class ListResponse extends $pb.GeneratedMessage {
  factory ListResponse({
    List_? clients,
    Error? error,
  }) {
    final result = create();
    if (clients != null) result.clients = clients;
    if (error != null) result.error = error;
    return result;
  }

  ListResponse._();

  factory ListResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ListResponse_Result>
      _ListResponse_ResultByTag = {
    1: ListResponse_Result.clients,
    2: ListResponse_Result.error,
    0: ListResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<List_>(1, _omitFieldNames ? '' : 'clients', subBuilder: List_.create)
    ..aE<Error>(2, _omitFieldNames ? '' : 'error', enumValues: Error.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListResponse copyWith(void Function(ListResponse) updates) =>
      super.copyWith((message) => updates(message as ListResponse))
          as ListResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListResponse create() => ListResponse._();
  @$core.override
  ListResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListResponse>(create);
  static ListResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  ListResponse_Result whichResult() =>
      _ListResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  List_ get clients => $_getN(0);
  @$pb.TagNumber(1)
  set clients(List_ value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasClients() => $_has(0);
  @$pb.TagNumber(1)
  void clearClients() => $_clearField(1);
  @$pb.TagNumber(1)
  List_ ensureClients() => $_ensure(0);

  @$pb.TagNumber(2)
  Error get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(Error value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

class ConnectionRequest extends $pb.GeneratedMessage {
  factory ConnectionRequest({
    $core.List<$core.int>? pubkey,
    $0.ClientInfo? info,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    if (info != null) result.info = info;
    return result;
  }

  ConnectionRequest._();

  factory ConnectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConnectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConnectionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..aOM<$0.ClientInfo>(2, _omitFieldNames ? '' : 'info',
        subBuilder: $0.ClientInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectionRequest copyWith(void Function(ConnectionRequest) updates) =>
      super.copyWith((message) => updates(message as ConnectionRequest))
          as ConnectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConnectionRequest create() => ConnectionRequest._();
  @$core.override
  ConnectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConnectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConnectionRequest>(create);
  static ConnectionRequest? _defaultInstance;

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

class ConnectionResponse extends $pb.GeneratedMessage {
  factory ConnectionResponse({
    $core.bool? approved,
    $core.List<$core.int>? pubkey,
  }) {
    final result = create();
    if (approved != null) result.approved = approved;
    if (pubkey != null) result.pubkey = pubkey;
    return result;
  }

  ConnectionResponse._();

  factory ConnectionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConnectionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConnectionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'approved')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectionResponse copyWith(void Function(ConnectionResponse) updates) =>
      super.copyWith((message) => updates(message as ConnectionResponse))
          as ConnectionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConnectionResponse create() => ConnectionResponse._();
  @$core.override
  ConnectionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConnectionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConnectionResponse>(create);
  static ConnectionResponse? _defaultInstance;

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

class ConnectionCancel extends $pb.GeneratedMessage {
  factory ConnectionCancel({
    $core.List<$core.int>? pubkey,
  }) {
    final result = create();
    if (pubkey != null) result.pubkey = pubkey;
    return result;
  }

  ConnectionCancel._();

  factory ConnectionCancel.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConnectionCancel.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConnectionCancel',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'pubkey', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectionCancel clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectionCancel copyWith(void Function(ConnectionCancel) updates) =>
      super.copyWith((message) => updates(message as ConnectionCancel))
          as ConnectionCancel;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConnectionCancel create() => ConnectionCancel._();
  @$core.override
  ConnectionCancel createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConnectionCancel getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConnectionCancel>(create);
  static ConnectionCancel? _defaultInstance;

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
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
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

class WalletAccessEntry extends $pb.GeneratedMessage {
  factory WalletAccessEntry({
    $core.int? id,
    WalletAccess? access,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (access != null) result.access = access;
    return result;
  }

  WalletAccessEntry._();

  factory WalletAccessEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WalletAccessEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WalletAccessEntry',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id')
    ..aOM<WalletAccess>(2, _omitFieldNames ? '' : 'access',
        subBuilder: WalletAccess.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletAccessEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletAccessEntry copyWith(void Function(WalletAccessEntry) updates) =>
      super.copyWith((message) => updates(message as WalletAccessEntry))
          as WalletAccessEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WalletAccessEntry create() => WalletAccessEntry._();
  @$core.override
  WalletAccessEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WalletAccessEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WalletAccessEntry>(create);
  static WalletAccessEntry? _defaultInstance;

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

class GrantWalletAccess extends $pb.GeneratedMessage {
  factory GrantWalletAccess({
    $core.Iterable<WalletAccess>? accesses,
  }) {
    final result = create();
    if (accesses != null) result.accesses.addAll(accesses);
    return result;
  }

  GrantWalletAccess._();

  factory GrantWalletAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GrantWalletAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GrantWalletAccess',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..pPM<WalletAccess>(1, _omitFieldNames ? '' : 'accesses',
        subBuilder: WalletAccess.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GrantWalletAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GrantWalletAccess copyWith(void Function(GrantWalletAccess) updates) =>
      super.copyWith((message) => updates(message as GrantWalletAccess))
          as GrantWalletAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GrantWalletAccess create() => GrantWalletAccess._();
  @$core.override
  GrantWalletAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GrantWalletAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GrantWalletAccess>(create);
  static GrantWalletAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WalletAccess> get accesses => $_getList(0);
}

class RevokeWalletAccess extends $pb.GeneratedMessage {
  factory RevokeWalletAccess({
    $core.Iterable<$core.int>? accesses,
  }) {
    final result = create();
    if (accesses != null) result.accesses.addAll(accesses);
    return result;
  }

  RevokeWalletAccess._();

  factory RevokeWalletAccess.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RevokeWalletAccess.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RevokeWalletAccess',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..p<$core.int>(1, _omitFieldNames ? '' : 'accesses', $pb.PbFieldType.K3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeWalletAccess clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RevokeWalletAccess copyWith(void Function(RevokeWalletAccess) updates) =>
      super.copyWith((message) => updates(message as RevokeWalletAccess))
          as RevokeWalletAccess;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RevokeWalletAccess create() => RevokeWalletAccess._();
  @$core.override
  RevokeWalletAccess createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RevokeWalletAccess getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RevokeWalletAccess>(create);
  static RevokeWalletAccess? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.int> get accesses => $_getList(0);
}

class ListWalletAccessResponse extends $pb.GeneratedMessage {
  factory ListWalletAccessResponse({
    $core.Iterable<WalletAccessEntry>? accesses,
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
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..pPM<WalletAccessEntry>(1, _omitFieldNames ? '' : 'accesses',
        subBuilder: WalletAccessEntry.create)
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
  $pb.PbList<WalletAccessEntry> get accesses => $_getList(0);
}

enum Request_Payload {
  connectionResponse,
  revoke,
  list,
  grantWalletAccess,
  revokeWalletAccess,
  listWalletAccess,
  notSet
}

class Request extends $pb.GeneratedMessage {
  factory Request({
    ConnectionResponse? connectionResponse,
    RevokeRequest? revoke,
    $1.Empty? list,
    GrantWalletAccess? grantWalletAccess,
    RevokeWalletAccess? revokeWalletAccess,
    $1.Empty? listWalletAccess,
  }) {
    final result = create();
    if (connectionResponse != null)
      result.connectionResponse = connectionResponse;
    if (revoke != null) result.revoke = revoke;
    if (list != null) result.list = list;
    if (grantWalletAccess != null) result.grantWalletAccess = grantWalletAccess;
    if (revokeWalletAccess != null)
      result.revokeWalletAccess = revokeWalletAccess;
    if (listWalletAccess != null) result.listWalletAccess = listWalletAccess;
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
    1: Request_Payload.connectionResponse,
    2: Request_Payload.revoke,
    3: Request_Payload.list,
    4: Request_Payload.grantWalletAccess,
    5: Request_Payload.revokeWalletAccess,
    6: Request_Payload.listWalletAccess,
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6])
    ..aOM<ConnectionResponse>(1, _omitFieldNames ? '' : 'connectionResponse',
        subBuilder: ConnectionResponse.create)
    ..aOM<RevokeRequest>(2, _omitFieldNames ? '' : 'revoke',
        subBuilder: RevokeRequest.create)
    ..aOM<$1.Empty>(3, _omitFieldNames ? '' : 'list',
        subBuilder: $1.Empty.create)
    ..aOM<GrantWalletAccess>(4, _omitFieldNames ? '' : 'grantWalletAccess',
        subBuilder: GrantWalletAccess.create)
    ..aOM<RevokeWalletAccess>(5, _omitFieldNames ? '' : 'revokeWalletAccess',
        subBuilder: RevokeWalletAccess.create)
    ..aOM<$1.Empty>(6, _omitFieldNames ? '' : 'listWalletAccess',
        subBuilder: $1.Empty.create)
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
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  Request_Payload whichPayload() => _Request_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  ConnectionResponse get connectionResponse => $_getN(0);
  @$pb.TagNumber(1)
  set connectionResponse(ConnectionResponse value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectionResponse() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectionResponse() => $_clearField(1);
  @$pb.TagNumber(1)
  ConnectionResponse ensureConnectionResponse() => $_ensure(0);

  @$pb.TagNumber(2)
  RevokeRequest get revoke => $_getN(1);
  @$pb.TagNumber(2)
  set revoke(RevokeRequest value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRevoke() => $_has(1);
  @$pb.TagNumber(2)
  void clearRevoke() => $_clearField(2);
  @$pb.TagNumber(2)
  RevokeRequest ensureRevoke() => $_ensure(1);

  @$pb.TagNumber(3)
  $1.Empty get list => $_getN(2);
  @$pb.TagNumber(3)
  set list($1.Empty value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasList() => $_has(2);
  @$pb.TagNumber(3)
  void clearList() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Empty ensureList() => $_ensure(2);

  @$pb.TagNumber(4)
  GrantWalletAccess get grantWalletAccess => $_getN(3);
  @$pb.TagNumber(4)
  set grantWalletAccess(GrantWalletAccess value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasGrantWalletAccess() => $_has(3);
  @$pb.TagNumber(4)
  void clearGrantWalletAccess() => $_clearField(4);
  @$pb.TagNumber(4)
  GrantWalletAccess ensureGrantWalletAccess() => $_ensure(3);

  @$pb.TagNumber(5)
  RevokeWalletAccess get revokeWalletAccess => $_getN(4);
  @$pb.TagNumber(5)
  set revokeWalletAccess(RevokeWalletAccess value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasRevokeWalletAccess() => $_has(4);
  @$pb.TagNumber(5)
  void clearRevokeWalletAccess() => $_clearField(5);
  @$pb.TagNumber(5)
  RevokeWalletAccess ensureRevokeWalletAccess() => $_ensure(4);

  @$pb.TagNumber(6)
  $1.Empty get listWalletAccess => $_getN(5);
  @$pb.TagNumber(6)
  set listWalletAccess($1.Empty value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasListWalletAccess() => $_has(5);
  @$pb.TagNumber(6)
  void clearListWalletAccess() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Empty ensureListWalletAccess() => $_ensure(5);
}

enum Response_Payload {
  connectionRequest,
  connectionCancel,
  revoke,
  list,
  listWalletAccess,
  notSet
}

class Response extends $pb.GeneratedMessage {
  factory Response({
    ConnectionRequest? connectionRequest,
    ConnectionCancel? connectionCancel,
    RevokeResponse? revoke,
    ListResponse? list,
    ListWalletAccessResponse? listWalletAccess,
  }) {
    final result = create();
    if (connectionRequest != null) result.connectionRequest = connectionRequest;
    if (connectionCancel != null) result.connectionCancel = connectionCancel;
    if (revoke != null) result.revoke = revoke;
    if (list != null) result.list = list;
    if (listWalletAccess != null) result.listWalletAccess = listWalletAccess;
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
    1: Response_Payload.connectionRequest,
    2: Response_Payload.connectionCancel,
    3: Response_Payload.revoke,
    4: Response_Payload.list,
    5: Response_Payload.listWalletAccess,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.sdk_client'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5])
    ..aOM<ConnectionRequest>(1, _omitFieldNames ? '' : 'connectionRequest',
        subBuilder: ConnectionRequest.create)
    ..aOM<ConnectionCancel>(2, _omitFieldNames ? '' : 'connectionCancel',
        subBuilder: ConnectionCancel.create)
    ..aOM<RevokeResponse>(3, _omitFieldNames ? '' : 'revoke',
        subBuilder: RevokeResponse.create)
    ..aOM<ListResponse>(4, _omitFieldNames ? '' : 'list',
        subBuilder: ListResponse.create)
    ..aOM<ListWalletAccessResponse>(
        5, _omitFieldNames ? '' : 'listWalletAccess',
        subBuilder: ListWalletAccessResponse.create)
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
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  ConnectionRequest get connectionRequest => $_getN(0);
  @$pb.TagNumber(1)
  set connectionRequest(ConnectionRequest value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectionRequest() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectionRequest() => $_clearField(1);
  @$pb.TagNumber(1)
  ConnectionRequest ensureConnectionRequest() => $_ensure(0);

  @$pb.TagNumber(2)
  ConnectionCancel get connectionCancel => $_getN(1);
  @$pb.TagNumber(2)
  set connectionCancel(ConnectionCancel value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasConnectionCancel() => $_has(1);
  @$pb.TagNumber(2)
  void clearConnectionCancel() => $_clearField(2);
  @$pb.TagNumber(2)
  ConnectionCancel ensureConnectionCancel() => $_ensure(1);

  @$pb.TagNumber(3)
  RevokeResponse get revoke => $_getN(2);
  @$pb.TagNumber(3)
  set revoke(RevokeResponse value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasRevoke() => $_has(2);
  @$pb.TagNumber(3)
  void clearRevoke() => $_clearField(3);
  @$pb.TagNumber(3)
  RevokeResponse ensureRevoke() => $_ensure(2);

  @$pb.TagNumber(4)
  ListResponse get list => $_getN(3);
  @$pb.TagNumber(4)
  set list(ListResponse value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasList() => $_has(3);
  @$pb.TagNumber(4)
  void clearList() => $_clearField(4);
  @$pb.TagNumber(4)
  ListResponse ensureList() => $_ensure(3);

  @$pb.TagNumber(5)
  ListWalletAccessResponse get listWalletAccess => $_getN(4);
  @$pb.TagNumber(5)
  set listWalletAccess(ListWalletAccessResponse value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasListWalletAccess() => $_has(4);
  @$pb.TagNumber(5)
  void clearListWalletAccess() => $_clearField(5);
  @$pb.TagNumber(5)
  ListWalletAccessResponse ensureListWalletAccess() => $_ensure(4);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
