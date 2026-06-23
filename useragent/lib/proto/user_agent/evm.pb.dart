// This is a generated file - do not edit.
//
// Generated from user_agent/evm.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $1;

import '../evm.pb.dart' as $0;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class SignTransactionRequest extends $pb.GeneratedMessage {
  factory SignTransactionRequest({
    $core.int? clientId,
    $0.EvmSignTransactionRequest? request,
  }) {
    final result = create();
    if (clientId != null) result.clientId = clientId;
    if (request != null) result.request = request;
    return result;
  }

  SignTransactionRequest._();

  factory SignTransactionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SignTransactionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SignTransactionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.evm'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'clientId')
    ..aOM<$0.EvmSignTransactionRequest>(2, _omitFieldNames ? '' : 'request',
        subBuilder: $0.EvmSignTransactionRequest.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SignTransactionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SignTransactionRequest copyWith(
          void Function(SignTransactionRequest) updates) =>
      super.copyWith((message) => updates(message as SignTransactionRequest))
          as SignTransactionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SignTransactionRequest create() => SignTransactionRequest._();
  @$core.override
  SignTransactionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SignTransactionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SignTransactionRequest>(create);
  static SignTransactionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get clientId => $_getIZ(0);
  @$pb.TagNumber(1)
  set clientId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);

  @$pb.TagNumber(2)
  $0.EvmSignTransactionRequest get request => $_getN(1);
  @$pb.TagNumber(2)
  set request($0.EvmSignTransactionRequest value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRequest() => $_has(1);
  @$pb.TagNumber(2)
  void clearRequest() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.EvmSignTransactionRequest ensureRequest() => $_ensure(1);
}

enum Request_Payload {
  walletCreate,
  walletList,
  grantCreate,
  grantDelete,
  grantList,
  signTransaction,
  notSet
}

class Request extends $pb.GeneratedMessage {
  factory Request({
    $1.Empty? walletCreate,
    $1.Empty? walletList,
    $0.EvmGrantCreateRequest? grantCreate,
    $0.EvmGrantDeleteRequest? grantDelete,
    $0.EvmGrantListRequest? grantList,
    SignTransactionRequest? signTransaction,
  }) {
    final result = create();
    if (walletCreate != null) result.walletCreate = walletCreate;
    if (walletList != null) result.walletList = walletList;
    if (grantCreate != null) result.grantCreate = grantCreate;
    if (grantDelete != null) result.grantDelete = grantDelete;
    if (grantList != null) result.grantList = grantList;
    if (signTransaction != null) result.signTransaction = signTransaction;
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
    1: Request_Payload.walletCreate,
    2: Request_Payload.walletList,
    3: Request_Payload.grantCreate,
    4: Request_Payload.grantDelete,
    5: Request_Payload.grantList,
    6: Request_Payload.signTransaction,
    0: Request_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Request',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6])
    ..aOM<$1.Empty>(1, _omitFieldNames ? '' : 'walletCreate',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(2, _omitFieldNames ? '' : 'walletList',
        subBuilder: $1.Empty.create)
    ..aOM<$0.EvmGrantCreateRequest>(3, _omitFieldNames ? '' : 'grantCreate',
        subBuilder: $0.EvmGrantCreateRequest.create)
    ..aOM<$0.EvmGrantDeleteRequest>(4, _omitFieldNames ? '' : 'grantDelete',
        subBuilder: $0.EvmGrantDeleteRequest.create)
    ..aOM<$0.EvmGrantListRequest>(5, _omitFieldNames ? '' : 'grantList',
        subBuilder: $0.EvmGrantListRequest.create)
    ..aOM<SignTransactionRequest>(6, _omitFieldNames ? '' : 'signTransaction',
        subBuilder: SignTransactionRequest.create)
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
  $1.Empty get walletCreate => $_getN(0);
  @$pb.TagNumber(1)
  set walletCreate($1.Empty value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletCreate() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletCreate() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Empty ensureWalletCreate() => $_ensure(0);

  @$pb.TagNumber(2)
  $1.Empty get walletList => $_getN(1);
  @$pb.TagNumber(2)
  set walletList($1.Empty value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasWalletList() => $_has(1);
  @$pb.TagNumber(2)
  void clearWalletList() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Empty ensureWalletList() => $_ensure(1);

  @$pb.TagNumber(3)
  $0.EvmGrantCreateRequest get grantCreate => $_getN(2);
  @$pb.TagNumber(3)
  set grantCreate($0.EvmGrantCreateRequest value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasGrantCreate() => $_has(2);
  @$pb.TagNumber(3)
  void clearGrantCreate() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.EvmGrantCreateRequest ensureGrantCreate() => $_ensure(2);

  @$pb.TagNumber(4)
  $0.EvmGrantDeleteRequest get grantDelete => $_getN(3);
  @$pb.TagNumber(4)
  set grantDelete($0.EvmGrantDeleteRequest value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasGrantDelete() => $_has(3);
  @$pb.TagNumber(4)
  void clearGrantDelete() => $_clearField(4);
  @$pb.TagNumber(4)
  $0.EvmGrantDeleteRequest ensureGrantDelete() => $_ensure(3);

  @$pb.TagNumber(5)
  $0.EvmGrantListRequest get grantList => $_getN(4);
  @$pb.TagNumber(5)
  set grantList($0.EvmGrantListRequest value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasGrantList() => $_has(4);
  @$pb.TagNumber(5)
  void clearGrantList() => $_clearField(5);
  @$pb.TagNumber(5)
  $0.EvmGrantListRequest ensureGrantList() => $_ensure(4);

  @$pb.TagNumber(6)
  SignTransactionRequest get signTransaction => $_getN(5);
  @$pb.TagNumber(6)
  set signTransaction(SignTransactionRequest value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasSignTransaction() => $_has(5);
  @$pb.TagNumber(6)
  void clearSignTransaction() => $_clearField(6);
  @$pb.TagNumber(6)
  SignTransactionRequest ensureSignTransaction() => $_ensure(5);
}

enum Response_Payload {
  walletCreate,
  walletList,
  grantCreate,
  grantDelete,
  grantList,
  signTransaction,
  notSet
}

class Response extends $pb.GeneratedMessage {
  factory Response({
    $0.WalletCreateResponse? walletCreate,
    $0.WalletListResponse? walletList,
    $0.EvmGrantCreateResponse? grantCreate,
    $0.EvmGrantDeleteResponse? grantDelete,
    $0.EvmGrantListResponse? grantList,
    $0.EvmSignTransactionResponse? signTransaction,
  }) {
    final result = create();
    if (walletCreate != null) result.walletCreate = walletCreate;
    if (walletList != null) result.walletList = walletList;
    if (grantCreate != null) result.grantCreate = grantCreate;
    if (grantDelete != null) result.grantDelete = grantDelete;
    if (grantList != null) result.grantList = grantList;
    if (signTransaction != null) result.signTransaction = signTransaction;
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
    1: Response_Payload.walletCreate,
    2: Response_Payload.walletList,
    3: Response_Payload.grantCreate,
    4: Response_Payload.grantDelete,
    5: Response_Payload.grantList,
    6: Response_Payload.signTransaction,
    0: Response_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Response',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'arbiter.user_agent.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6])
    ..aOM<$0.WalletCreateResponse>(1, _omitFieldNames ? '' : 'walletCreate',
        subBuilder: $0.WalletCreateResponse.create)
    ..aOM<$0.WalletListResponse>(2, _omitFieldNames ? '' : 'walletList',
        subBuilder: $0.WalletListResponse.create)
    ..aOM<$0.EvmGrantCreateResponse>(3, _omitFieldNames ? '' : 'grantCreate',
        subBuilder: $0.EvmGrantCreateResponse.create)
    ..aOM<$0.EvmGrantDeleteResponse>(4, _omitFieldNames ? '' : 'grantDelete',
        subBuilder: $0.EvmGrantDeleteResponse.create)
    ..aOM<$0.EvmGrantListResponse>(5, _omitFieldNames ? '' : 'grantList',
        subBuilder: $0.EvmGrantListResponse.create)
    ..aOM<$0.EvmSignTransactionResponse>(
        6, _omitFieldNames ? '' : 'signTransaction',
        subBuilder: $0.EvmSignTransactionResponse.create)
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
  @$pb.TagNumber(6)
  Response_Payload whichPayload() => _Response_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $0.WalletCreateResponse get walletCreate => $_getN(0);
  @$pb.TagNumber(1)
  set walletCreate($0.WalletCreateResponse value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletCreate() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletCreate() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.WalletCreateResponse ensureWalletCreate() => $_ensure(0);

  @$pb.TagNumber(2)
  $0.WalletListResponse get walletList => $_getN(1);
  @$pb.TagNumber(2)
  set walletList($0.WalletListResponse value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasWalletList() => $_has(1);
  @$pb.TagNumber(2)
  void clearWalletList() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.WalletListResponse ensureWalletList() => $_ensure(1);

  @$pb.TagNumber(3)
  $0.EvmGrantCreateResponse get grantCreate => $_getN(2);
  @$pb.TagNumber(3)
  set grantCreate($0.EvmGrantCreateResponse value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasGrantCreate() => $_has(2);
  @$pb.TagNumber(3)
  void clearGrantCreate() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.EvmGrantCreateResponse ensureGrantCreate() => $_ensure(2);

  @$pb.TagNumber(4)
  $0.EvmGrantDeleteResponse get grantDelete => $_getN(3);
  @$pb.TagNumber(4)
  set grantDelete($0.EvmGrantDeleteResponse value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasGrantDelete() => $_has(3);
  @$pb.TagNumber(4)
  void clearGrantDelete() => $_clearField(4);
  @$pb.TagNumber(4)
  $0.EvmGrantDeleteResponse ensureGrantDelete() => $_ensure(3);

  @$pb.TagNumber(5)
  $0.EvmGrantListResponse get grantList => $_getN(4);
  @$pb.TagNumber(5)
  set grantList($0.EvmGrantListResponse value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasGrantList() => $_has(4);
  @$pb.TagNumber(5)
  void clearGrantList() => $_clearField(5);
  @$pb.TagNumber(5)
  $0.EvmGrantListResponse ensureGrantList() => $_ensure(4);

  @$pb.TagNumber(6)
  $0.EvmSignTransactionResponse get signTransaction => $_getN(5);
  @$pb.TagNumber(6)
  set signTransaction($0.EvmSignTransactionResponse value) =>
      $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasSignTransaction() => $_has(5);
  @$pb.TagNumber(6)
  void clearSignTransaction() => $_clearField(6);
  @$pb.TagNumber(6)
  $0.EvmSignTransactionResponse ensureSignTransaction() => $_ensure(5);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
