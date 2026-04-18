// This is a generated file - do not edit.
//
// Generated from shared/evm.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $0;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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

class EvalViolation_ChainIdMismatch extends $pb.GeneratedMessage {
  factory EvalViolation_ChainIdMismatch({
    $fixnum.Int64? expected,
    $fixnum.Int64? actual,
  }) {
    final result = create();
    if (expected != null) result.expected = expected;
    if (actual != null) result.actual = actual;
    return result;
  }

  EvalViolation_ChainIdMismatch._();

  factory EvalViolation_ChainIdMismatch.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvalViolation_ChainIdMismatch.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvalViolation.ChainIdMismatch',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'expected', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'actual', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvalViolation_ChainIdMismatch clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvalViolation_ChainIdMismatch copyWith(
          void Function(EvalViolation_ChainIdMismatch) updates) =>
      super.copyWith(
              (message) => updates(message as EvalViolation_ChainIdMismatch))
          as EvalViolation_ChainIdMismatch;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvalViolation_ChainIdMismatch create() =>
      EvalViolation_ChainIdMismatch._();
  @$core.override
  EvalViolation_ChainIdMismatch createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvalViolation_ChainIdMismatch getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvalViolation_ChainIdMismatch>(create);
  static EvalViolation_ChainIdMismatch? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get expected => $_getI64(0);
  @$pb.TagNumber(1)
  set expected($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExpected() => $_has(0);
  @$pb.TagNumber(1)
  void clearExpected() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get actual => $_getI64(1);
  @$pb.TagNumber(2)
  set actual($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasActual() => $_has(1);
  @$pb.TagNumber(2)
  void clearActual() => $_clearField(2);
}

enum EvalViolation_Kind {
  invalidTarget,
  gasLimitExceeded,
  rateLimitExceeded,
  volumetricLimitExceeded,
  invalidTime,
  invalidTransactionType,
  chainIdMismatch,
  notSet
}

class EvalViolation extends $pb.GeneratedMessage {
  factory EvalViolation({
    $core.List<$core.int>? invalidTarget,
    GasLimitExceededViolation? gasLimitExceeded,
    $0.Empty? rateLimitExceeded,
    $0.Empty? volumetricLimitExceeded,
    $0.Empty? invalidTime,
    $0.Empty? invalidTransactionType,
    EvalViolation_ChainIdMismatch? chainIdMismatch,
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
    if (chainIdMismatch != null) result.chainIdMismatch = chainIdMismatch;
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
    7: EvalViolation_Kind.chainIdMismatch,
    0: EvalViolation_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvalViolation',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7])
    ..a<$core.List<$core.int>>(
        1, _omitFieldNames ? '' : 'invalidTarget', $pb.PbFieldType.OY)
    ..aOM<GasLimitExceededViolation>(
        2, _omitFieldNames ? '' : 'gasLimitExceeded',
        subBuilder: GasLimitExceededViolation.create)
    ..aOM<$0.Empty>(3, _omitFieldNames ? '' : 'rateLimitExceeded',
        subBuilder: $0.Empty.create)
    ..aOM<$0.Empty>(4, _omitFieldNames ? '' : 'volumetricLimitExceeded',
        subBuilder: $0.Empty.create)
    ..aOM<$0.Empty>(5, _omitFieldNames ? '' : 'invalidTime',
        subBuilder: $0.Empty.create)
    ..aOM<$0.Empty>(6, _omitFieldNames ? '' : 'invalidTransactionType',
        subBuilder: $0.Empty.create)
    ..aOM<EvalViolation_ChainIdMismatch>(
        7, _omitFieldNames ? '' : 'chainIdMismatch',
        subBuilder: EvalViolation_ChainIdMismatch.create)
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
  @$pb.TagNumber(7)
  EvalViolation_Kind whichKind() => _EvalViolation_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
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
  $0.Empty get rateLimitExceeded => $_getN(2);
  @$pb.TagNumber(3)
  set rateLimitExceeded($0.Empty value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasRateLimitExceeded() => $_has(2);
  @$pb.TagNumber(3)
  void clearRateLimitExceeded() => $_clearField(3);
  @$pb.TagNumber(3)
  $0.Empty ensureRateLimitExceeded() => $_ensure(2);

  @$pb.TagNumber(4)
  $0.Empty get volumetricLimitExceeded => $_getN(3);
  @$pb.TagNumber(4)
  set volumetricLimitExceeded($0.Empty value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasVolumetricLimitExceeded() => $_has(3);
  @$pb.TagNumber(4)
  void clearVolumetricLimitExceeded() => $_clearField(4);
  @$pb.TagNumber(4)
  $0.Empty ensureVolumetricLimitExceeded() => $_ensure(3);

  @$pb.TagNumber(5)
  $0.Empty get invalidTime => $_getN(4);
  @$pb.TagNumber(5)
  set invalidTime($0.Empty value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasInvalidTime() => $_has(4);
  @$pb.TagNumber(5)
  void clearInvalidTime() => $_clearField(5);
  @$pb.TagNumber(5)
  $0.Empty ensureInvalidTime() => $_ensure(4);

  @$pb.TagNumber(6)
  $0.Empty get invalidTransactionType => $_getN(5);
  @$pb.TagNumber(6)
  set invalidTransactionType($0.Empty value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasInvalidTransactionType() => $_has(5);
  @$pb.TagNumber(6)
  void clearInvalidTransactionType() => $_clearField(6);
  @$pb.TagNumber(6)
  $0.Empty ensureInvalidTransactionType() => $_ensure(5);

  @$pb.TagNumber(7)
  EvalViolation_ChainIdMismatch get chainIdMismatch => $_getN(6);
  @$pb.TagNumber(7)
  set chainIdMismatch(EvalViolation_ChainIdMismatch value) =>
      $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasChainIdMismatch() => $_has(6);
  @$pb.TagNumber(7)
  void clearChainIdMismatch() => $_clearField(7);
  @$pb.TagNumber(7)
  EvalViolation_ChainIdMismatch ensureChainIdMismatch() => $_ensure(6);
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
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
    $0.Empty? contractCreationNotSupported,
    $0.Empty? unsupportedTransactionType,
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'arbiter.shared.evm'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOM<$0.Empty>(1, _omitFieldNames ? '' : 'contractCreationNotSupported',
        subBuilder: $0.Empty.create)
    ..aOM<$0.Empty>(2, _omitFieldNames ? '' : 'unsupportedTransactionType',
        subBuilder: $0.Empty.create)
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
  $0.Empty get contractCreationNotSupported => $_getN(0);
  @$pb.TagNumber(1)
  set contractCreationNotSupported($0.Empty value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasContractCreationNotSupported() => $_has(0);
  @$pb.TagNumber(1)
  void clearContractCreationNotSupported() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.Empty ensureContractCreationNotSupported() => $_ensure(0);

  @$pb.TagNumber(2)
  $0.Empty get unsupportedTransactionType => $_getN(1);
  @$pb.TagNumber(2)
  set unsupportedTransactionType($0.Empty value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasUnsupportedTransactionType() => $_has(1);
  @$pb.TagNumber(2)
  void clearUnsupportedTransactionType() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.Empty ensureUnsupportedTransactionType() => $_ensure(1);

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

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
