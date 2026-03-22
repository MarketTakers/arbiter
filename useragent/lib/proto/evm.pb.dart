// This is a generated file - do not edit.
//
// Generated from evm.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $1;
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart'
    as $0;

import 'evm.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'evm.pbenum.dart';

class WalletEntry extends $pb.GeneratedMessage {
  factory WalletEntry({
    $core.List<$core.int>? address,
  }) {
    final result = create();
    if (address != null) result.address = address;
    return result;
  }

  WalletEntry._();

  factory WalletEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WalletEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WalletEntry',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'address', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletEntry copyWith(void Function(WalletEntry) updates) =>
      super.copyWith((message) => updates(message as WalletEntry))
          as WalletEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WalletEntry create() => WalletEntry._();
  @$core.override
  WalletEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WalletEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WalletEntry>(create);
  static WalletEntry? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get address => $_getN(0);
  @$pb.TagNumber(1)
  set address($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAddress() => $_has(0);
  @$pb.TagNumber(1)
  void clearAddress() => $_clearField(1);
}

class WalletList extends $pb.GeneratedMessage {
  factory WalletList({
    $core.Iterable<WalletEntry>? wallets,
  }) {
    final result = create();
    if (wallets != null) result.wallets.addAll(wallets);
    return result;
  }

  WalletList._();

  factory WalletList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WalletList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WalletList',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..pPM<WalletEntry>(1, _omitFieldNames ? '' : 'wallets',
        subBuilder: WalletEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletList copyWith(void Function(WalletList) updates) =>
      super.copyWith((message) => updates(message as WalletList)) as WalletList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WalletList create() => WalletList._();
  @$core.override
  WalletList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WalletList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WalletList>(create);
  static WalletList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WalletEntry> get wallets => $_getList(0);
}

enum WalletCreateResponse_Result { wallet, error, notSet }

class WalletCreateResponse extends $pb.GeneratedMessage {
  factory WalletCreateResponse({
    WalletEntry? wallet,
    EvmError? error,
  }) {
    final result = create();
    if (wallet != null) result.wallet = wallet;
    if (error != null) result.error = error;
    return result;
  }

  WalletCreateResponse._();

  factory WalletCreateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WalletCreateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, WalletCreateResponse_Result>
      _WalletCreateResponse_ResultByTag = {
    1: WalletCreateResponse_Result.wallet,
    2: WalletCreateResponse_Result.error,
    0: WalletCreateResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WalletCreateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<WalletEntry>(1, _omitFieldNames ? '' : 'wallet',
        subBuilder: WalletEntry.create)
    ..aE<EvmError>(2, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletCreateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletCreateResponse copyWith(void Function(WalletCreateResponse) updates) =>
      super.copyWith((message) => updates(message as WalletCreateResponse))
          as WalletCreateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WalletCreateResponse create() => WalletCreateResponse._();
  @$core.override
  WalletCreateResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WalletCreateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WalletCreateResponse>(create);
  static WalletCreateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  WalletCreateResponse_Result whichResult() =>
      _WalletCreateResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  WalletEntry get wallet => $_getN(0);
  @$pb.TagNumber(1)
  set wallet(WalletEntry value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWallet() => $_has(0);
  @$pb.TagNumber(1)
  void clearWallet() => $_clearField(1);
  @$pb.TagNumber(1)
  WalletEntry ensureWallet() => $_ensure(0);

  @$pb.TagNumber(2)
  EvmError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(EvmError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

enum WalletListResponse_Result { wallets, error, notSet }

class WalletListResponse extends $pb.GeneratedMessage {
  factory WalletListResponse({
    WalletList? wallets,
    EvmError? error,
  }) {
    final result = create();
    if (wallets != null) result.wallets = wallets;
    if (error != null) result.error = error;
    return result;
  }

  WalletListResponse._();

  factory WalletListResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WalletListResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, WalletListResponse_Result>
      _WalletListResponse_ResultByTag = {
    1: WalletListResponse_Result.wallets,
    2: WalletListResponse_Result.error,
    0: WalletListResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WalletListResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<WalletList>(1, _omitFieldNames ? '' : 'wallets',
        subBuilder: WalletList.create)
    ..aE<EvmError>(2, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletListResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WalletListResponse copyWith(void Function(WalletListResponse) updates) =>
      super.copyWith((message) => updates(message as WalletListResponse))
          as WalletListResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WalletListResponse create() => WalletListResponse._();
  @$core.override
  WalletListResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WalletListResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WalletListResponse>(create);
  static WalletListResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  WalletListResponse_Result whichResult() =>
      _WalletListResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  WalletList get wallets => $_getN(0);
  @$pb.TagNumber(1)
  set wallets(WalletList value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWallets() => $_has(0);
  @$pb.TagNumber(1)
  void clearWallets() => $_clearField(1);
  @$pb.TagNumber(1)
  WalletList ensureWallets() => $_ensure(0);

  @$pb.TagNumber(2)
  EvmError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(EvmError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

class TransactionRateLimit extends $pb.GeneratedMessage {
  factory TransactionRateLimit({
    $core.int? count,
    $fixnum.Int64? windowSecs,
  }) {
    final result = create();
    if (count != null) result.count = count;
    if (windowSecs != null) result.windowSecs = windowSecs;
    return result;
  }

  TransactionRateLimit._();

  factory TransactionRateLimit.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TransactionRateLimit.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TransactionRateLimit',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'count', fieldType: $pb.PbFieldType.OU3)
    ..aInt64(2, _omitFieldNames ? '' : 'windowSecs')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TransactionRateLimit clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TransactionRateLimit copyWith(void Function(TransactionRateLimit) updates) =>
      super.copyWith((message) => updates(message as TransactionRateLimit))
          as TransactionRateLimit;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TransactionRateLimit create() => TransactionRateLimit._();
  @$core.override
  TransactionRateLimit createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TransactionRateLimit getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TransactionRateLimit>(create);
  static TransactionRateLimit? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get count => $_getIZ(0);
  @$pb.TagNumber(1)
  set count($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get windowSecs => $_getI64(1);
  @$pb.TagNumber(2)
  set windowSecs($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWindowSecs() => $_has(1);
  @$pb.TagNumber(2)
  void clearWindowSecs() => $_clearField(2);
}

class VolumeRateLimit extends $pb.GeneratedMessage {
  factory VolumeRateLimit({
    $core.List<$core.int>? maxVolume,
    $fixnum.Int64? windowSecs,
  }) {
    final result = create();
    if (maxVolume != null) result.maxVolume = maxVolume;
    if (windowSecs != null) result.windowSecs = windowSecs;
    return result;
  }

  VolumeRateLimit._();

  factory VolumeRateLimit.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VolumeRateLimit.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VolumeRateLimit',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'maxVolume', $pb.PbFieldType.OY)
    ..aInt64(2, _omitFieldNames ? '' : 'windowSecs')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VolumeRateLimit clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VolumeRateLimit copyWith(void Function(VolumeRateLimit) updates) =>
      super.copyWith((message) => updates(message as VolumeRateLimit))
          as VolumeRateLimit;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VolumeRateLimit create() => VolumeRateLimit._();
  @$core.override
  VolumeRateLimit createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VolumeRateLimit getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VolumeRateLimit>(create);
  static VolumeRateLimit? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get maxVolume => $_getN(0);
  @$pb.TagNumber(1)
  set maxVolume($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMaxVolume() => $_has(0);
  @$pb.TagNumber(1)
  void clearMaxVolume() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get windowSecs => $_getI64(1);
  @$pb.TagNumber(2)
  set windowSecs($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWindowSecs() => $_has(1);
  @$pb.TagNumber(2)
  void clearWindowSecs() => $_clearField(2);
}

class SharedSettings extends $pb.GeneratedMessage {
  factory SharedSettings({
    $core.int? walletAccessId,
    $fixnum.Int64? chainId,
    $0.Timestamp? validFrom,
    $0.Timestamp? validUntil,
    $core.List<$core.int>? maxGasFeePerGas,
    $core.List<$core.int>? maxPriorityFeePerGas,
    TransactionRateLimit? rateLimit,
  }) {
    final result = create();
    if (walletAccessId != null) result.walletAccessId = walletAccessId;
    if (chainId != null) result.chainId = chainId;
    if (validFrom != null) result.validFrom = validFrom;
    if (validUntil != null) result.validUntil = validUntil;
    if (maxGasFeePerGas != null) result.maxGasFeePerGas = maxGasFeePerGas;
    if (maxPriorityFeePerGas != null)
      result.maxPriorityFeePerGas = maxPriorityFeePerGas;
    if (rateLimit != null) result.rateLimit = rateLimit;
    return result;
  }

  SharedSettings._();

  factory SharedSettings.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SharedSettings.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SharedSettings',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'walletAccessId')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'chainId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<$0.Timestamp>(3, _omitFieldNames ? '' : 'validFrom',
        subBuilder: $0.Timestamp.create)
    ..aOM<$0.Timestamp>(4, _omitFieldNames ? '' : 'validUntil',
        subBuilder: $0.Timestamp.create)
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'maxGasFeePerGas', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        6, _omitFieldNames ? '' : 'maxPriorityFeePerGas', $pb.PbFieldType.OY)
    ..aOM<TransactionRateLimit>(7, _omitFieldNames ? '' : 'rateLimit',
        subBuilder: TransactionRateLimit.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SharedSettings clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SharedSettings copyWith(void Function(SharedSettings) updates) =>
      super.copyWith((message) => updates(message as SharedSettings))
          as SharedSettings;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SharedSettings create() => SharedSettings._();
  @$core.override
  SharedSettings createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SharedSettings getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SharedSettings>(create);
  static SharedSettings? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get walletAccessId => $_getIZ(0);
  @$pb.TagNumber(1)
  set walletAccessId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletAccessId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletAccessId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get chainId => $_getI64(1);
  @$pb.TagNumber(2)
  set chainId($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasChainId() => $_has(1);
  @$pb.TagNumber(2)
  void clearChainId() => $_clearField(2);

  @$pb.TagNumber(3)
  $0.Timestamp get validFrom => $_getN(2);
  @$pb.TagNumber(3)
  set validFrom($0.Timestamp value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasValidFrom() => $_has(2);
  @$pb.TagNumber(3)
  void clearValidFrom() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.Timestamp ensureValidFrom() => $_ensure(2);

  @$pb.TagNumber(4)
  $0.Timestamp get validUntil => $_getN(3);
  @$pb.TagNumber(4)
  set validUntil($0.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasValidUntil() => $_has(3);
  @$pb.TagNumber(4)
  void clearValidUntil() => $_clearField(4);
  @$pb.TagNumber(4)
  $0.Timestamp ensureValidUntil() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.List<$core.int> get maxGasFeePerGas => $_getN(4);
  @$pb.TagNumber(5)
  set maxGasFeePerGas($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMaxGasFeePerGas() => $_has(4);
  @$pb.TagNumber(5)
  void clearMaxGasFeePerGas() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.List<$core.int> get maxPriorityFeePerGas => $_getN(5);
  @$pb.TagNumber(6)
  set maxPriorityFeePerGas($core.List<$core.int> value) => $_setBytes(5, value);
  @$pb.TagNumber(6)
  $core.bool hasMaxPriorityFeePerGas() => $_has(5);
  @$pb.TagNumber(6)
  void clearMaxPriorityFeePerGas() => $_clearField(6);

  @$pb.TagNumber(7)
  TransactionRateLimit get rateLimit => $_getN(6);
  @$pb.TagNumber(7)
  set rateLimit(TransactionRateLimit value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasRateLimit() => $_has(6);
  @$pb.TagNumber(7)
  void clearRateLimit() => $_clearField(7);
  @$pb.TagNumber(7)
  TransactionRateLimit ensureRateLimit() => $_ensure(6);
}

class EtherTransferSettings extends $pb.GeneratedMessage {
  factory EtherTransferSettings({
    $core.Iterable<$core.List<$core.int>>? targets,
    VolumeRateLimit? limit,
  }) {
    final result = create();
    if (targets != null) result.targets.addAll(targets);
    if (limit != null) result.limit = limit;
    return result;
  }

  EtherTransferSettings._();

  factory EtherTransferSettings.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EtherTransferSettings.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EtherTransferSettings',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..p<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'targets', $pb.PbFieldType.PY)
    ..aOM<VolumeRateLimit>(2, _omitFieldNames ? '' : 'limit',
        subBuilder: VolumeRateLimit.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EtherTransferSettings clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EtherTransferSettings copyWith(
          void Function(EtherTransferSettings) updates) =>
      super.copyWith((message) => updates(message as EtherTransferSettings))
          as EtherTransferSettings;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EtherTransferSettings create() => EtherTransferSettings._();
  @$core.override
  EtherTransferSettings createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EtherTransferSettings getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EtherTransferSettings>(create);
  static EtherTransferSettings? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.List<$core.int>> get targets => $_getList(0);

  @$pb.TagNumber(2)
  VolumeRateLimit get limit => $_getN(1);
  @$pb.TagNumber(2)
  set limit(VolumeRateLimit value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasLimit() => $_has(1);
  @$pb.TagNumber(2)
  void clearLimit() => $_clearField(2);
  @$pb.TagNumber(2)
  VolumeRateLimit ensureLimit() => $_ensure(1);
}

class TokenTransferSettings extends $pb.GeneratedMessage {
  factory TokenTransferSettings({
    $core.List<$core.int>? tokenContract,
    $core.List<$core.int>? target,
    $core.Iterable<VolumeRateLimit>? volumeLimits,
  }) {
    final result = create();
    if (tokenContract != null) result.tokenContract = tokenContract;
    if (target != null) result.target = target;
    if (volumeLimits != null) result.volumeLimits.addAll(volumeLimits);
    return result;
  }

  TokenTransferSettings._();

  factory TokenTransferSettings.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenTransferSettings.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenTransferSettings',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'tokenContract', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'target', $pb.PbFieldType.OY)
    ..pPM<VolumeRateLimit>(3, _omitFieldNames ? '' : 'volumeLimits',
        subBuilder: VolumeRateLimit.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenTransferSettings clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenTransferSettings copyWith(
          void Function(TokenTransferSettings) updates) =>
      super.copyWith((message) => updates(message as TokenTransferSettings))
          as TokenTransferSettings;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenTransferSettings create() => TokenTransferSettings._();
  @$core.override
  TokenTransferSettings createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenTransferSettings getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TokenTransferSettings>(create);
  static TokenTransferSettings? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get tokenContract => $_getN(0);
  @$pb.TagNumber(1)
  set tokenContract($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTokenContract() => $_has(0);
  @$pb.TagNumber(1)
  void clearTokenContract() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get target => $_getN(1);
  @$pb.TagNumber(2)
  set target($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTarget() => $_has(1);
  @$pb.TagNumber(2)
  void clearTarget() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<VolumeRateLimit> get volumeLimits => $_getList(2);
}

enum SpecificGrant_Grant { etherTransfer, tokenTransfer, notSet }

class SpecificGrant extends $pb.GeneratedMessage {
  factory SpecificGrant({
    EtherTransferSettings? etherTransfer,
    TokenTransferSettings? tokenTransfer,
  }) {
    final result = create();
    if (etherTransfer != null) result.etherTransfer = etherTransfer;
    if (tokenTransfer != null) result.tokenTransfer = tokenTransfer;
    return result;
  }

  SpecificGrant._();

  factory SpecificGrant.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SpecificGrant.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SpecificGrant_Grant>
      _SpecificGrant_GrantByTag = {
    1: SpecificGrant_Grant.etherTransfer,
    2: SpecificGrant_Grant.tokenTransfer,
    0: SpecificGrant_Grant.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SpecificGrant',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<EtherTransferSettings>(1, _omitFieldNames ? '' : 'etherTransfer',
        subBuilder: EtherTransferSettings.create)
    ..aOM<TokenTransferSettings>(2, _omitFieldNames ? '' : 'tokenTransfer',
        subBuilder: TokenTransferSettings.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SpecificGrant clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SpecificGrant copyWith(void Function(SpecificGrant) updates) =>
      super.copyWith((message) => updates(message as SpecificGrant))
          as SpecificGrant;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SpecificGrant create() => SpecificGrant._();
  @$core.override
  SpecificGrant createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SpecificGrant getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SpecificGrant>(create);
  static SpecificGrant? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  SpecificGrant_Grant whichGrant() =>
      _SpecificGrant_GrantByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearGrant() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  EtherTransferSettings get etherTransfer => $_getN(0);
  @$pb.TagNumber(1)
  set etherTransfer(EtherTransferSettings value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasEtherTransfer() => $_has(0);
  @$pb.TagNumber(1)
  void clearEtherTransfer() => $_clearField(1);
  @$pb.TagNumber(1)
  EtherTransferSettings ensureEtherTransfer() => $_ensure(0);

  @$pb.TagNumber(2)
  TokenTransferSettings get tokenTransfer => $_getN(1);
  @$pb.TagNumber(2)
  set tokenTransfer(TokenTransferSettings value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasTokenTransfer() => $_has(1);
  @$pb.TagNumber(2)
  void clearTokenTransfer() => $_clearField(2);
  @$pb.TagNumber(2)
  TokenTransferSettings ensureTokenTransfer() => $_ensure(1);
}

class EtherTransferMeaning extends $pb.GeneratedMessage {
  factory EtherTransferMeaning({
    $core.List<$core.int>? to,
    $core.List<$core.int>? value,
  }) {
    final result = create();
    if (to != null) result.to = to;
    if (value != null) result.value = value;
    return result;
  }

  EtherTransferMeaning._();

  factory EtherTransferMeaning.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EtherTransferMeaning.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EtherTransferMeaning',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'to', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'value', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EtherTransferMeaning clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EtherTransferMeaning copyWith(void Function(EtherTransferMeaning) updates) =>
      super.copyWith((message) => updates(message as EtherTransferMeaning))
          as EtherTransferMeaning;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EtherTransferMeaning create() => EtherTransferMeaning._();
  @$core.override
  EtherTransferMeaning createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EtherTransferMeaning getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EtherTransferMeaning>(create);
  static EtherTransferMeaning? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get to => $_getN(0);
  @$pb.TagNumber(1)
  set to($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTo() => $_has(0);
  @$pb.TagNumber(1)
  void clearTo() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get value => $_getN(1);
  @$pb.TagNumber(2)
  set value($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);
}

class TokenInfo extends $pb.GeneratedMessage {
  factory TokenInfo({
    $core.String? symbol,
    $core.List<$core.int>? address,
    $fixnum.Int64? chainId,
  }) {
    final result = create();
    if (symbol != null) result.symbol = symbol;
    if (address != null) result.address = address;
    if (chainId != null) result.chainId = chainId;
    return result;
  }

  TokenInfo._();

  factory TokenInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenInfo',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'symbol')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'address', $pb.PbFieldType.OY)
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'chainId', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenInfo copyWith(void Function(TokenInfo) updates) =>
      super.copyWith((message) => updates(message as TokenInfo)) as TokenInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenInfo create() => TokenInfo._();
  @$core.override
  TokenInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenInfo getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TokenInfo>(create);
  static TokenInfo? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get symbol => $_getSZ(0);
  @$pb.TagNumber(1)
  set symbol($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSymbol() => $_has(0);
  @$pb.TagNumber(1)
  void clearSymbol() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get address => $_getN(1);
  @$pb.TagNumber(2)
  set address($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAddress() => $_has(1);
  @$pb.TagNumber(2)
  void clearAddress() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get chainId => $_getI64(2);
  @$pb.TagNumber(3)
  set chainId($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasChainId() => $_has(2);
  @$pb.TagNumber(3)
  void clearChainId() => $_clearField(3);
}

/// Mirror of token_transfers::Meaning
class TokenTransferMeaning extends $pb.GeneratedMessage {
  factory TokenTransferMeaning({
    TokenInfo? token,
    $core.List<$core.int>? to,
    $core.List<$core.int>? value,
  }) {
    final result = create();
    if (token != null) result.token = token;
    if (to != null) result.to = to;
    if (value != null) result.value = value;
    return result;
  }

  TokenTransferMeaning._();

  factory TokenTransferMeaning.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TokenTransferMeaning.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TokenTransferMeaning',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aOM<TokenInfo>(1, _omitFieldNames ? '' : 'token',
        subBuilder: TokenInfo.create)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'to', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'value', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenTransferMeaning clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TokenTransferMeaning copyWith(void Function(TokenTransferMeaning) updates) =>
      super.copyWith((message) => updates(message as TokenTransferMeaning))
          as TokenTransferMeaning;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TokenTransferMeaning create() => TokenTransferMeaning._();
  @$core.override
  TokenTransferMeaning createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TokenTransferMeaning getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TokenTransferMeaning>(create);
  static TokenTransferMeaning? _defaultInstance;

  @$pb.TagNumber(1)
  TokenInfo get token => $_getN(0);
  @$pb.TagNumber(1)
  set token(TokenInfo value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasToken() => $_has(0);
  @$pb.TagNumber(1)
  void clearToken() => $_clearField(1);
  @$pb.TagNumber(1)
  TokenInfo ensureToken() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.List<$core.int> get to => $_getN(1);
  @$pb.TagNumber(2)
  set to($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTo() => $_has(1);
  @$pb.TagNumber(2)
  void clearTo() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.List<$core.int> get value => $_getN(2);
  @$pb.TagNumber(3)
  set value($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);
}

enum SpecificMeaning_Meaning { etherTransfer, tokenTransfer, notSet }

/// Mirror of policies::SpecificMeaning
class SpecificMeaning extends $pb.GeneratedMessage {
  factory SpecificMeaning({
    EtherTransferMeaning? etherTransfer,
    TokenTransferMeaning? tokenTransfer,
  }) {
    final result = create();
    if (etherTransfer != null) result.etherTransfer = etherTransfer;
    if (tokenTransfer != null) result.tokenTransfer = tokenTransfer;
    return result;
  }

  SpecificMeaning._();

  factory SpecificMeaning.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SpecificMeaning.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SpecificMeaning_Meaning>
      _SpecificMeaning_MeaningByTag = {
    1: SpecificMeaning_Meaning.etherTransfer,
    2: SpecificMeaning_Meaning.tokenTransfer,
    0: SpecificMeaning_Meaning.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SpecificMeaning',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<EtherTransferMeaning>(1, _omitFieldNames ? '' : 'etherTransfer',
        subBuilder: EtherTransferMeaning.create)
    ..aOM<TokenTransferMeaning>(2, _omitFieldNames ? '' : 'tokenTransfer',
        subBuilder: TokenTransferMeaning.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SpecificMeaning clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SpecificMeaning copyWith(void Function(SpecificMeaning) updates) =>
      super.copyWith((message) => updates(message as SpecificMeaning))
          as SpecificMeaning;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SpecificMeaning create() => SpecificMeaning._();
  @$core.override
  SpecificMeaning createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SpecificMeaning getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SpecificMeaning>(create);
  static SpecificMeaning? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  SpecificMeaning_Meaning whichMeaning() =>
      _SpecificMeaning_MeaningByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearMeaning() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  EtherTransferMeaning get etherTransfer => $_getN(0);
  @$pb.TagNumber(1)
  set etherTransfer(EtherTransferMeaning value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasEtherTransfer() => $_has(0);
  @$pb.TagNumber(1)
  void clearEtherTransfer() => $_clearField(1);
  @$pb.TagNumber(1)
  EtherTransferMeaning ensureEtherTransfer() => $_ensure(0);

  @$pb.TagNumber(2)
  TokenTransferMeaning get tokenTransfer => $_getN(1);
  @$pb.TagNumber(2)
  set tokenTransfer(TokenTransferMeaning value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasTokenTransfer() => $_has(1);
  @$pb.TagNumber(2)
  void clearTokenTransfer() => $_clearField(2);
  @$pb.TagNumber(2)
  TokenTransferMeaning ensureTokenTransfer() => $_ensure(1);
}

/// --- Eval error types ---
class GasLimitExceededViolation extends $pb.GeneratedMessage {
  factory GasLimitExceededViolation({
    $core.List<$core.int>? maxGasFeePerGas,
    $core.List<$core.int>? maxPriorityFeePerGas,
  }) {
    final result = create();
    if (maxGasFeePerGas != null) result.maxGasFeePerGas = maxGasFeePerGas;
    if (maxPriorityFeePerGas != null)
      result.maxPriorityFeePerGas = maxPriorityFeePerGas;
    return result;
  }

  GasLimitExceededViolation._();

  factory GasLimitExceededViolation.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GasLimitExceededViolation.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GasLimitExceededViolation',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'maxGasFeePerGas', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'maxPriorityFeePerGas', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GasLimitExceededViolation clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GasLimitExceededViolation copyWith(
          void Function(GasLimitExceededViolation) updates) =>
      super.copyWith((message) => updates(message as GasLimitExceededViolation))
          as GasLimitExceededViolation;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GasLimitExceededViolation create() => GasLimitExceededViolation._();
  @$core.override
  GasLimitExceededViolation createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GasLimitExceededViolation getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GasLimitExceededViolation>(create);
  static GasLimitExceededViolation? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get maxGasFeePerGas => $_getN(0);
  @$pb.TagNumber(1)
  set maxGasFeePerGas($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMaxGasFeePerGas() => $_has(0);
  @$pb.TagNumber(1)
  void clearMaxGasFeePerGas() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get maxPriorityFeePerGas => $_getN(1);
  @$pb.TagNumber(2)
  set maxPriorityFeePerGas($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMaxPriorityFeePerGas() => $_has(1);
  @$pb.TagNumber(2)
  void clearMaxPriorityFeePerGas() => $_clearField(2);
}

enum EvalViolation_Kind {
  invalidTarget,
  gasLimitExceeded,
  rateLimitExceeded,
  volumetricLimitExceeded,
  invalidTime,
  invalidTransactionType,
  notSet
}

class EvalViolation extends $pb.GeneratedMessage {
  factory EvalViolation({
    $core.List<$core.int>? invalidTarget,
    GasLimitExceededViolation? gasLimitExceeded,
    $1.Empty? rateLimitExceeded,
    $1.Empty? volumetricLimitExceeded,
    $1.Empty? invalidTime,
    $1.Empty? invalidTransactionType,
  }) {
    final result = create();
    if (invalidTarget != null) result.invalidTarget = invalidTarget;
    if (gasLimitExceeded != null) result.gasLimitExceeded = gasLimitExceeded;
    if (rateLimitExceeded != null) result.rateLimitExceeded = rateLimitExceeded;
    if (volumetricLimitExceeded != null)
      result.volumetricLimitExceeded = volumetricLimitExceeded;
    if (invalidTime != null) result.invalidTime = invalidTime;
    if (invalidTransactionType != null)
      result.invalidTransactionType = invalidTransactionType;
    return result;
  }

  EvalViolation._();

  factory EvalViolation.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvalViolation.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EvalViolation_Kind>
      _EvalViolation_KindByTag = {
    1: EvalViolation_Kind.invalidTarget,
    2: EvalViolation_Kind.gasLimitExceeded,
    3: EvalViolation_Kind.rateLimitExceeded,
    4: EvalViolation_Kind.volumetricLimitExceeded,
    5: EvalViolation_Kind.invalidTime,
    6: EvalViolation_Kind.invalidTransactionType,
    0: EvalViolation_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvalViolation',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6])
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'invalidTarget', $pb.PbFieldType.OY)
    ..aOM<GasLimitExceededViolation>(
        2, _omitFieldNames ? '' : 'gasLimitExceeded',
        subBuilder: GasLimitExceededViolation.create)
    ..aOM<$1.Empty>(3, _omitFieldNames ? '' : 'rateLimitExceeded',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(4, _omitFieldNames ? '' : 'volumetricLimitExceeded',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(5, _omitFieldNames ? '' : 'invalidTime',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(6, _omitFieldNames ? '' : 'invalidTransactionType',
        subBuilder: $1.Empty.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvalViolation clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvalViolation copyWith(void Function(EvalViolation) updates) =>
      super.copyWith((message) => updates(message as EvalViolation))
          as EvalViolation;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvalViolation create() => EvalViolation._();
  @$core.override
  EvalViolation createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvalViolation getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvalViolation>(create);
  static EvalViolation? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  EvalViolation_Kind whichKind() => _EvalViolation_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.List<$core.int> get invalidTarget => $_getN(0);
  @$pb.TagNumber(1)
  set invalidTarget($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInvalidTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearInvalidTarget() => $_clearField(1);

  @$pb.TagNumber(2)
  GasLimitExceededViolation get gasLimitExceeded => $_getN(1);
  @$pb.TagNumber(2)
  set gasLimitExceeded(GasLimitExceededViolation value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasGasLimitExceeded() => $_has(1);
  @$pb.TagNumber(2)
  void clearGasLimitExceeded() => $_clearField(2);
  @$pb.TagNumber(2)
  GasLimitExceededViolation ensureGasLimitExceeded() => $_ensure(1);

  @$pb.TagNumber(3)
  $1.Empty get rateLimitExceeded => $_getN(2);
  @$pb.TagNumber(3)
  set rateLimitExceeded($1.Empty value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasRateLimitExceeded() => $_has(2);
  @$pb.TagNumber(3)
  void clearRateLimitExceeded() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Empty ensureRateLimitExceeded() => $_ensure(2);

  @$pb.TagNumber(4)
  $1.Empty get volumetricLimitExceeded => $_getN(3);
  @$pb.TagNumber(4)
  set volumetricLimitExceeded($1.Empty value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasVolumetricLimitExceeded() => $_has(3);
  @$pb.TagNumber(4)
  void clearVolumetricLimitExceeded() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Empty ensureVolumetricLimitExceeded() => $_ensure(3);

  @$pb.TagNumber(5)
  $1.Empty get invalidTime => $_getN(4);
  @$pb.TagNumber(5)
  set invalidTime($1.Empty value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasInvalidTime() => $_has(4);
  @$pb.TagNumber(5)
  void clearInvalidTime() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Empty ensureInvalidTime() => $_ensure(4);

  @$pb.TagNumber(6)
  $1.Empty get invalidTransactionType => $_getN(5);
  @$pb.TagNumber(6)
  set invalidTransactionType($1.Empty value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasInvalidTransactionType() => $_has(5);
  @$pb.TagNumber(6)
  void clearInvalidTransactionType() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Empty ensureInvalidTransactionType() => $_ensure(5);
}

/// Transaction was classified but no grant covers it
class NoMatchingGrantError extends $pb.GeneratedMessage {
  factory NoMatchingGrantError({
    SpecificMeaning? meaning,
  }) {
    final result = create();
    if (meaning != null) result.meaning = meaning;
    return result;
  }

  NoMatchingGrantError._();

  factory NoMatchingGrantError.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory NoMatchingGrantError.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'NoMatchingGrantError',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aOM<SpecificMeaning>(1, _omitFieldNames ? '' : 'meaning',
        subBuilder: SpecificMeaning.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NoMatchingGrantError clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  NoMatchingGrantError copyWith(void Function(NoMatchingGrantError) updates) =>
      super.copyWith((message) => updates(message as NoMatchingGrantError))
          as NoMatchingGrantError;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static NoMatchingGrantError create() => NoMatchingGrantError._();
  @$core.override
  NoMatchingGrantError createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static NoMatchingGrantError getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<NoMatchingGrantError>(create);
  static NoMatchingGrantError? _defaultInstance;

  @$pb.TagNumber(1)
  SpecificMeaning get meaning => $_getN(0);
  @$pb.TagNumber(1)
  set meaning(SpecificMeaning value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMeaning() => $_has(0);
  @$pb.TagNumber(1)
  void clearMeaning() => $_clearField(1);
  @$pb.TagNumber(1)
  SpecificMeaning ensureMeaning() => $_ensure(0);
}

/// Transaction was classified and a grant was found, but constraints were violated
class PolicyViolationsError extends $pb.GeneratedMessage {
  factory PolicyViolationsError({
    SpecificMeaning? meaning,
    $core.Iterable<EvalViolation>? violations,
  }) {
    final result = create();
    if (meaning != null) result.meaning = meaning;
    if (violations != null) result.violations.addAll(violations);
    return result;
  }

  PolicyViolationsError._();

  factory PolicyViolationsError.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PolicyViolationsError.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PolicyViolationsError',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aOM<SpecificMeaning>(1, _omitFieldNames ? '' : 'meaning',
        subBuilder: SpecificMeaning.create)
    ..pPM<EvalViolation>(2, _omitFieldNames ? '' : 'violations',
        subBuilder: EvalViolation.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PolicyViolationsError clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PolicyViolationsError copyWith(
          void Function(PolicyViolationsError) updates) =>
      super.copyWith((message) => updates(message as PolicyViolationsError))
          as PolicyViolationsError;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PolicyViolationsError create() => PolicyViolationsError._();
  @$core.override
  PolicyViolationsError createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PolicyViolationsError getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PolicyViolationsError>(create);
  static PolicyViolationsError? _defaultInstance;

  @$pb.TagNumber(1)
  SpecificMeaning get meaning => $_getN(0);
  @$pb.TagNumber(1)
  set meaning(SpecificMeaning value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMeaning() => $_has(0);
  @$pb.TagNumber(1)
  void clearMeaning() => $_clearField(1);
  @$pb.TagNumber(1)
  SpecificMeaning ensureMeaning() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<EvalViolation> get violations => $_getList(1);
}

enum TransactionEvalError_Kind {
  contractCreationNotSupported,
  unsupportedTransactionType,
  noMatchingGrant,
  policyViolations,
  notSet
}

/// top-level error returned when transaction evaluation fails
class TransactionEvalError extends $pb.GeneratedMessage {
  factory TransactionEvalError({
    $1.Empty? contractCreationNotSupported,
    $1.Empty? unsupportedTransactionType,
    NoMatchingGrantError? noMatchingGrant,
    PolicyViolationsError? policyViolations,
  }) {
    final result = create();
    if (contractCreationNotSupported != null)
      result.contractCreationNotSupported = contractCreationNotSupported;
    if (unsupportedTransactionType != null)
      result.unsupportedTransactionType = unsupportedTransactionType;
    if (noMatchingGrant != null) result.noMatchingGrant = noMatchingGrant;
    if (policyViolations != null) result.policyViolations = policyViolations;
    return result;
  }

  TransactionEvalError._();

  factory TransactionEvalError.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TransactionEvalError.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, TransactionEvalError_Kind>
      _TransactionEvalError_KindByTag = {
    1: TransactionEvalError_Kind.contractCreationNotSupported,
    2: TransactionEvalError_Kind.unsupportedTransactionType,
    3: TransactionEvalError_Kind.noMatchingGrant,
    4: TransactionEvalError_Kind.policyViolations,
    0: TransactionEvalError_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TransactionEvalError',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOM<$1.Empty>(1, _omitFieldNames ? '' : 'contractCreationNotSupported',
        subBuilder: $1.Empty.create)
    ..aOM<$1.Empty>(2, _omitFieldNames ? '' : 'unsupportedTransactionType',
        subBuilder: $1.Empty.create)
    ..aOM<NoMatchingGrantError>(3, _omitFieldNames ? '' : 'noMatchingGrant',
        subBuilder: NoMatchingGrantError.create)
    ..aOM<PolicyViolationsError>(4, _omitFieldNames ? '' : 'policyViolations',
        subBuilder: PolicyViolationsError.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TransactionEvalError clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TransactionEvalError copyWith(void Function(TransactionEvalError) updates) =>
      super.copyWith((message) => updates(message as TransactionEvalError))
          as TransactionEvalError;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TransactionEvalError create() => TransactionEvalError._();
  @$core.override
  TransactionEvalError createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TransactionEvalError getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TransactionEvalError>(create);
  static TransactionEvalError? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  TransactionEvalError_Kind whichKind() =>
      _TransactionEvalError_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $1.Empty get contractCreationNotSupported => $_getN(0);
  @$pb.TagNumber(1)
  set contractCreationNotSupported($1.Empty value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasContractCreationNotSupported() => $_has(0);
  @$pb.TagNumber(1)
  void clearContractCreationNotSupported() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Empty ensureContractCreationNotSupported() => $_ensure(0);

  @$pb.TagNumber(2)
  $1.Empty get unsupportedTransactionType => $_getN(1);
  @$pb.TagNumber(2)
  set unsupportedTransactionType($1.Empty value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasUnsupportedTransactionType() => $_has(1);
  @$pb.TagNumber(2)
  void clearUnsupportedTransactionType() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Empty ensureUnsupportedTransactionType() => $_ensure(1);

  @$pb.TagNumber(3)
  NoMatchingGrantError get noMatchingGrant => $_getN(2);
  @$pb.TagNumber(3)
  set noMatchingGrant(NoMatchingGrantError value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasNoMatchingGrant() => $_has(2);
  @$pb.TagNumber(3)
  void clearNoMatchingGrant() => $_clearField(3);
  @$pb.TagNumber(3)
  NoMatchingGrantError ensureNoMatchingGrant() => $_ensure(2);

  @$pb.TagNumber(4)
  PolicyViolationsError get policyViolations => $_getN(3);
  @$pb.TagNumber(4)
  set policyViolations(PolicyViolationsError value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasPolicyViolations() => $_has(3);
  @$pb.TagNumber(4)
  void clearPolicyViolations() => $_clearField(4);
  @$pb.TagNumber(4)
  PolicyViolationsError ensurePolicyViolations() => $_ensure(3);
}

/// --- UserAgent grant management ---
class EvmGrantCreateRequest extends $pb.GeneratedMessage {
  factory EvmGrantCreateRequest({
    SharedSettings? shared,
    SpecificGrant? specific,
  }) {
    final result = create();
    if (shared != null) result.shared = shared;
    if (specific != null) result.specific = specific;
    return result;
  }

  EvmGrantCreateRequest._();

  factory EvmGrantCreateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantCreateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantCreateRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aOM<SharedSettings>(1, _omitFieldNames ? '' : 'shared',
        subBuilder: SharedSettings.create)
    ..aOM<SpecificGrant>(2, _omitFieldNames ? '' : 'specific',
        subBuilder: SpecificGrant.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantCreateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantCreateRequest copyWith(
          void Function(EvmGrantCreateRequest) updates) =>
      super.copyWith((message) => updates(message as EvmGrantCreateRequest))
          as EvmGrantCreateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantCreateRequest create() => EvmGrantCreateRequest._();
  @$core.override
  EvmGrantCreateRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantCreateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantCreateRequest>(create);
  static EvmGrantCreateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  SharedSettings get shared => $_getN(0);
  @$pb.TagNumber(1)
  set shared(SharedSettings value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasShared() => $_has(0);
  @$pb.TagNumber(1)
  void clearShared() => $_clearField(1);
  @$pb.TagNumber(1)
  SharedSettings ensureShared() => $_ensure(0);

  @$pb.TagNumber(2)
  SpecificGrant get specific => $_getN(1);
  @$pb.TagNumber(2)
  set specific(SpecificGrant value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasSpecific() => $_has(1);
  @$pb.TagNumber(2)
  void clearSpecific() => $_clearField(2);
  @$pb.TagNumber(2)
  SpecificGrant ensureSpecific() => $_ensure(1);
}

enum EvmGrantCreateResponse_Result { grantId, error, notSet }

class EvmGrantCreateResponse extends $pb.GeneratedMessage {
  factory EvmGrantCreateResponse({
    $core.int? grantId,
    EvmError? error,
  }) {
    final result = create();
    if (grantId != null) result.grantId = grantId;
    if (error != null) result.error = error;
    return result;
  }

  EvmGrantCreateResponse._();

  factory EvmGrantCreateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantCreateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EvmGrantCreateResponse_Result>
      _EvmGrantCreateResponse_ResultByTag = {
    1: EvmGrantCreateResponse_Result.grantId,
    2: EvmGrantCreateResponse_Result.error,
    0: EvmGrantCreateResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantCreateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aI(1, _omitFieldNames ? '' : 'grantId')
    ..aE<EvmError>(2, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantCreateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantCreateResponse copyWith(
          void Function(EvmGrantCreateResponse) updates) =>
      super.copyWith((message) => updates(message as EvmGrantCreateResponse))
          as EvmGrantCreateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantCreateResponse create() => EvmGrantCreateResponse._();
  @$core.override
  EvmGrantCreateResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantCreateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantCreateResponse>(create);
  static EvmGrantCreateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  EvmGrantCreateResponse_Result whichResult() =>
      _EvmGrantCreateResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.int get grantId => $_getIZ(0);
  @$pb.TagNumber(1)
  set grantId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasGrantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearGrantId() => $_clearField(1);

  @$pb.TagNumber(2)
  EvmError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(EvmError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

class EvmGrantDeleteRequest extends $pb.GeneratedMessage {
  factory EvmGrantDeleteRequest({
    $core.int? grantId,
  }) {
    final result = create();
    if (grantId != null) result.grantId = grantId;
    return result;
  }

  EvmGrantDeleteRequest._();

  factory EvmGrantDeleteRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantDeleteRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantDeleteRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'grantId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantDeleteRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantDeleteRequest copyWith(
          void Function(EvmGrantDeleteRequest) updates) =>
      super.copyWith((message) => updates(message as EvmGrantDeleteRequest))
          as EvmGrantDeleteRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantDeleteRequest create() => EvmGrantDeleteRequest._();
  @$core.override
  EvmGrantDeleteRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantDeleteRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantDeleteRequest>(create);
  static EvmGrantDeleteRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get grantId => $_getIZ(0);
  @$pb.TagNumber(1)
  set grantId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasGrantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearGrantId() => $_clearField(1);
}

enum EvmGrantDeleteResponse_Result { ok, error, notSet }

class EvmGrantDeleteResponse extends $pb.GeneratedMessage {
  factory EvmGrantDeleteResponse({
    $1.Empty? ok,
    EvmError? error,
  }) {
    final result = create();
    if (ok != null) result.ok = ok;
    if (error != null) result.error = error;
    return result;
  }

  EvmGrantDeleteResponse._();

  factory EvmGrantDeleteResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantDeleteResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EvmGrantDeleteResponse_Result>
      _EvmGrantDeleteResponse_ResultByTag = {
    1: EvmGrantDeleteResponse_Result.ok,
    2: EvmGrantDeleteResponse_Result.error,
    0: EvmGrantDeleteResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantDeleteResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<$1.Empty>(1, _omitFieldNames ? '' : 'ok', subBuilder: $1.Empty.create)
    ..aE<EvmError>(2, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantDeleteResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantDeleteResponse copyWith(
          void Function(EvmGrantDeleteResponse) updates) =>
      super.copyWith((message) => updates(message as EvmGrantDeleteResponse))
          as EvmGrantDeleteResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantDeleteResponse create() => EvmGrantDeleteResponse._();
  @$core.override
  EvmGrantDeleteResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantDeleteResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantDeleteResponse>(create);
  static EvmGrantDeleteResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  EvmGrantDeleteResponse_Result whichResult() =>
      _EvmGrantDeleteResponse_ResultByTag[$_whichOneof(0)]!;
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
  EvmError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(EvmError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

/// Basic grant info returned in grant listings
class GrantEntry extends $pb.GeneratedMessage {
  factory GrantEntry({
    $core.int? id,
    $core.int? walletAccessId,
    SharedSettings? shared,
    SpecificGrant? specific,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (walletAccessId != null) result.walletAccessId = walletAccessId;
    if (shared != null) result.shared = shared;
    if (specific != null) result.specific = specific;
    return result;
  }

  GrantEntry._();

  factory GrantEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GrantEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GrantEntry',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id')
    ..aI(2, _omitFieldNames ? '' : 'walletAccessId')
    ..aOM<SharedSettings>(3, _omitFieldNames ? '' : 'shared',
        subBuilder: SharedSettings.create)
    ..aOM<SpecificGrant>(4, _omitFieldNames ? '' : 'specific',
        subBuilder: SpecificGrant.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GrantEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GrantEntry copyWith(void Function(GrantEntry) updates) =>
      super.copyWith((message) => updates(message as GrantEntry)) as GrantEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GrantEntry create() => GrantEntry._();
  @$core.override
  GrantEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GrantEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GrantEntry>(create);
  static GrantEntry? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get walletAccessId => $_getIZ(1);
  @$pb.TagNumber(2)
  set walletAccessId($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWalletAccessId() => $_has(1);
  @$pb.TagNumber(2)
  void clearWalletAccessId() => $_clearField(2);

  @$pb.TagNumber(3)
  SharedSettings get shared => $_getN(2);
  @$pb.TagNumber(3)
  set shared(SharedSettings value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasShared() => $_has(2);
  @$pb.TagNumber(3)
  void clearShared() => $_clearField(3);
  @$pb.TagNumber(3)
  SharedSettings ensureShared() => $_ensure(2);

  @$pb.TagNumber(4)
  SpecificGrant get specific => $_getN(3);
  @$pb.TagNumber(4)
  set specific(SpecificGrant value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSpecific() => $_has(3);
  @$pb.TagNumber(4)
  void clearSpecific() => $_clearField(4);
  @$pb.TagNumber(4)
  SpecificGrant ensureSpecific() => $_ensure(3);
}

class EvmGrantListRequest extends $pb.GeneratedMessage {
  factory EvmGrantListRequest({
    $core.int? walletAccessId,
  }) {
    final result = create();
    if (walletAccessId != null) result.walletAccessId = walletAccessId;
    return result;
  }

  EvmGrantListRequest._();

  factory EvmGrantListRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantListRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantListRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'walletAccessId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantListRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantListRequest copyWith(void Function(EvmGrantListRequest) updates) =>
      super.copyWith((message) => updates(message as EvmGrantListRequest))
          as EvmGrantListRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantListRequest create() => EvmGrantListRequest._();
  @$core.override
  EvmGrantListRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantListRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantListRequest>(create);
  static EvmGrantListRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get walletAccessId => $_getIZ(0);
  @$pb.TagNumber(1)
  set walletAccessId($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletAccessId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletAccessId() => $_clearField(1);
}

enum EvmGrantListResponse_Result { grants, error, notSet }

class EvmGrantListResponse extends $pb.GeneratedMessage {
  factory EvmGrantListResponse({
    EvmGrantList? grants,
    EvmError? error,
  }) {
    final result = create();
    if (grants != null) result.grants = grants;
    if (error != null) result.error = error;
    return result;
  }

  EvmGrantListResponse._();

  factory EvmGrantListResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantListResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EvmGrantListResponse_Result>
      _EvmGrantListResponse_ResultByTag = {
    1: EvmGrantListResponse_Result.grants,
    2: EvmGrantListResponse_Result.error,
    0: EvmGrantListResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantListResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2])
    ..aOM<EvmGrantList>(1, _omitFieldNames ? '' : 'grants',
        subBuilder: EvmGrantList.create)
    ..aE<EvmError>(2, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantListResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantListResponse copyWith(void Function(EvmGrantListResponse) updates) =>
      super.copyWith((message) => updates(message as EvmGrantListResponse))
          as EvmGrantListResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantListResponse create() => EvmGrantListResponse._();
  @$core.override
  EvmGrantListResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantListResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantListResponse>(create);
  static EvmGrantListResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  EvmGrantListResponse_Result whichResult() =>
      _EvmGrantListResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  EvmGrantList get grants => $_getN(0);
  @$pb.TagNumber(1)
  set grants(EvmGrantList value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasGrants() => $_has(0);
  @$pb.TagNumber(1)
  void clearGrants() => $_clearField(1);
  @$pb.TagNumber(1)
  EvmGrantList ensureGrants() => $_ensure(0);

  @$pb.TagNumber(2)
  EvmError get error => $_getN(1);
  @$pb.TagNumber(2)
  set error(EvmError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasError() => $_has(1);
  @$pb.TagNumber(2)
  void clearError() => $_clearField(2);
}

class EvmGrantList extends $pb.GeneratedMessage {
  factory EvmGrantList({
    $core.Iterable<GrantEntry>? grants,
  }) {
    final result = create();
    if (grants != null) result.grants.addAll(grants);
    return result;
  }

  EvmGrantList._();

  factory EvmGrantList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmGrantList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmGrantList',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..pPM<GrantEntry>(1, _omitFieldNames ? '' : 'grants',
        subBuilder: GrantEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmGrantList copyWith(void Function(EvmGrantList) updates) =>
      super.copyWith((message) => updates(message as EvmGrantList))
          as EvmGrantList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmGrantList create() => EvmGrantList._();
  @$core.override
  EvmGrantList createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmGrantList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmGrantList>(create);
  static EvmGrantList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GrantEntry> get grants => $_getList(0);
}

class EvmSignTransactionRequest extends $pb.GeneratedMessage {
  factory EvmSignTransactionRequest({
    $core.List<$core.int>? walletAddress,
    $core.List<$core.int>? rlpTransaction,
  }) {
    final result = create();
    if (walletAddress != null) result.walletAddress = walletAddress;
    if (rlpTransaction != null) result.rlpTransaction = rlpTransaction;
    return result;
  }

  EvmSignTransactionRequest._();

  factory EvmSignTransactionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmSignTransactionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmSignTransactionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'walletAddress', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'rlpTransaction', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmSignTransactionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmSignTransactionRequest copyWith(
          void Function(EvmSignTransactionRequest) updates) =>
      super.copyWith((message) => updates(message as EvmSignTransactionRequest))
          as EvmSignTransactionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmSignTransactionRequest create() => EvmSignTransactionRequest._();
  @$core.override
  EvmSignTransactionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmSignTransactionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmSignTransactionRequest>(create);
  static EvmSignTransactionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get walletAddress => $_getN(0);
  @$pb.TagNumber(1)
  set walletAddress($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletAddress() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletAddress() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get rlpTransaction => $_getN(1);
  @$pb.TagNumber(2)
  set rlpTransaction($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRlpTransaction() => $_has(1);
  @$pb.TagNumber(2)
  void clearRlpTransaction() => $_clearField(2);
}

enum EvmSignTransactionResponse_Result { signature, evalError, error, notSet }

/// oneof because signing and evaluation happen atomically — a signing failure
/// is always either an eval error or an internal error, never a partial success
class EvmSignTransactionResponse extends $pb.GeneratedMessage {
  factory EvmSignTransactionResponse({
    $core.List<$core.int>? signature,
    TransactionEvalError? evalError,
    EvmError? error,
  }) {
    final result = create();
    if (signature != null) result.signature = signature;
    if (evalError != null) result.evalError = evalError;
    if (error != null) result.error = error;
    return result;
  }

  EvmSignTransactionResponse._();

  factory EvmSignTransactionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmSignTransactionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EvmSignTransactionResponse_Result>
      _EvmSignTransactionResponse_ResultByTag = {
    1: EvmSignTransactionResponse_Result.signature,
    2: EvmSignTransactionResponse_Result.evalError,
    3: EvmSignTransactionResponse_Result.error,
    0: EvmSignTransactionResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmSignTransactionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'signature', $pb.PbFieldType.OY)
    ..aOM<TransactionEvalError>(2, _omitFieldNames ? '' : 'evalError',
        subBuilder: TransactionEvalError.create)
    ..aE<EvmError>(3, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmSignTransactionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmSignTransactionResponse copyWith(
          void Function(EvmSignTransactionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as EvmSignTransactionResponse))
          as EvmSignTransactionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmSignTransactionResponse create() => EvmSignTransactionResponse._();
  @$core.override
  EvmSignTransactionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmSignTransactionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmSignTransactionResponse>(create);
  static EvmSignTransactionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  EvmSignTransactionResponse_Result whichResult() =>
      _EvmSignTransactionResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.List<$core.int> get signature => $_getN(0);
  @$pb.TagNumber(1)
  set signature($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSignature() => $_has(0);
  @$pb.TagNumber(1)
  void clearSignature() => $_clearField(1);

  @$pb.TagNumber(2)
  TransactionEvalError get evalError => $_getN(1);
  @$pb.TagNumber(2)
  set evalError(TransactionEvalError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEvalError() => $_has(1);
  @$pb.TagNumber(2)
  void clearEvalError() => $_clearField(2);
  @$pb.TagNumber(2)
  TransactionEvalError ensureEvalError() => $_ensure(1);

  @$pb.TagNumber(3)
  EvmError get error => $_getN(2);
  @$pb.TagNumber(3)
  set error(EvmError value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasError() => $_has(2);
  @$pb.TagNumber(3)
  void clearError() => $_clearField(3);
}

class EvmAnalyzeTransactionRequest extends $pb.GeneratedMessage {
  factory EvmAnalyzeTransactionRequest({
    $core.List<$core.int>? walletAddress,
    $core.List<$core.int>? rlpTransaction,
  }) {
    final result = create();
    if (walletAddress != null) result.walletAddress = walletAddress;
    if (rlpTransaction != null) result.rlpTransaction = rlpTransaction;
    return result;
  }

  EvmAnalyzeTransactionRequest._();

  factory EvmAnalyzeTransactionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmAnalyzeTransactionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmAnalyzeTransactionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'walletAddress', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'rlpTransaction', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmAnalyzeTransactionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmAnalyzeTransactionRequest copyWith(
          void Function(EvmAnalyzeTransactionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as EvmAnalyzeTransactionRequest))
          as EvmAnalyzeTransactionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmAnalyzeTransactionRequest create() =>
      EvmAnalyzeTransactionRequest._();
  @$core.override
  EvmAnalyzeTransactionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmAnalyzeTransactionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmAnalyzeTransactionRequest>(create);
  static EvmAnalyzeTransactionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get walletAddress => $_getN(0);
  @$pb.TagNumber(1)
  set walletAddress($core.List<$core.int> value) => $_setBytes(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWalletAddress() => $_has(0);
  @$pb.TagNumber(1)
  void clearWalletAddress() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get rlpTransaction => $_getN(1);
  @$pb.TagNumber(2)
  set rlpTransaction($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRlpTransaction() => $_has(1);
  @$pb.TagNumber(2)
  void clearRlpTransaction() => $_clearField(2);
}

enum EvmAnalyzeTransactionResponse_Result { meaning, evalError, error, notSet }

class EvmAnalyzeTransactionResponse extends $pb.GeneratedMessage {
  factory EvmAnalyzeTransactionResponse({
    SpecificMeaning? meaning,
    TransactionEvalError? evalError,
    EvmError? error,
  }) {
    final result = create();
    if (meaning != null) result.meaning = meaning;
    if (evalError != null) result.evalError = evalError;
    if (error != null) result.error = error;
    return result;
  }

  EvmAnalyzeTransactionResponse._();

  factory EvmAnalyzeTransactionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvmAnalyzeTransactionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EvmAnalyzeTransactionResponse_Result>
      _EvmAnalyzeTransactionResponse_ResultByTag = {
    1: EvmAnalyzeTransactionResponse_Result.meaning,
    2: EvmAnalyzeTransactionResponse_Result.evalError,
    3: EvmAnalyzeTransactionResponse_Result.error,
    0: EvmAnalyzeTransactionResponse_Result.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvmAnalyzeTransactionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3])
    ..aOM<SpecificMeaning>(1, _omitFieldNames ? '' : 'meaning',
        subBuilder: SpecificMeaning.create)
    ..aOM<TransactionEvalError>(2, _omitFieldNames ? '' : 'evalError',
        subBuilder: TransactionEvalError.create)
    ..aE<EvmError>(3, _omitFieldNames ? '' : 'error',
        enumValues: EvmError.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmAnalyzeTransactionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvmAnalyzeTransactionResponse copyWith(
          void Function(EvmAnalyzeTransactionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as EvmAnalyzeTransactionResponse))
          as EvmAnalyzeTransactionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvmAnalyzeTransactionResponse create() =>
      EvmAnalyzeTransactionResponse._();
  @$core.override
  EvmAnalyzeTransactionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvmAnalyzeTransactionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvmAnalyzeTransactionResponse>(create);
  static EvmAnalyzeTransactionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  EvmAnalyzeTransactionResponse_Result whichResult() =>
      _EvmAnalyzeTransactionResponse_ResultByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearResult() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  SpecificMeaning get meaning => $_getN(0);
  @$pb.TagNumber(1)
  set meaning(SpecificMeaning value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMeaning() => $_has(0);
  @$pb.TagNumber(1)
  void clearMeaning() => $_clearField(1);
  @$pb.TagNumber(1)
  SpecificMeaning ensureMeaning() => $_ensure(0);

  @$pb.TagNumber(2)
  TransactionEvalError get evalError => $_getN(1);
  @$pb.TagNumber(2)
  set evalError(TransactionEvalError value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasEvalError() => $_has(1);
  @$pb.TagNumber(2)
  void clearEvalError() => $_clearField(2);
  @$pb.TagNumber(2)
  TransactionEvalError ensureEvalError() => $_ensure(1);

  @$pb.TagNumber(3)
  EvmError get error => $_getN(2);
  @$pb.TagNumber(3)
  set error(EvmError value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasError() => $_has(2);
  @$pb.TagNumber(3)
  void clearError() => $_clearField(3);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
