// This is a generated file - do not edit.
//
// Generated from evm.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use evmErrorDescriptor instead')
const EvmError$json = {
  '1': 'EvmError',
  '2': [
    {'1': 'EVM_ERROR_UNSPECIFIED', '2': 0},
    {'1': 'EVM_ERROR_VAULT_SEALED', '2': 1},
    {'1': 'EVM_ERROR_INTERNAL', '2': 2},
  ],
};

/// Descriptor for `EvmError`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List evmErrorDescriptor = $convert.base64Decode(
    'CghFdm1FcnJvchIZChVFVk1fRVJST1JfVU5TUEVDSUZJRUQQABIaChZFVk1fRVJST1JfVkFVTF'
    'RfU0VBTEVEEAESFgoSRVZNX0VSUk9SX0lOVEVSTkFMEAI=');

@$core.Deprecated('Use walletEntryDescriptor instead')
const WalletEntry$json = {
  '1': 'WalletEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 5, '10': 'id'},
    {'1': 'address', '3': 2, '4': 1, '5': 12, '10': 'address'},
  ],
};

/// Descriptor for `WalletEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List walletEntryDescriptor = $convert.base64Decode(
    'CgtXYWxsZXRFbnRyeRIOCgJpZBgBIAEoBVICaWQSGAoHYWRkcmVzcxgCIAEoDFIHYWRkcmVzcw'
    '==');

@$core.Deprecated('Use walletListDescriptor instead')
const WalletList$json = {
  '1': 'WalletList',
  '2': [
    {
      '1': 'wallets',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.arbiter.evm.WalletEntry',
      '10': 'wallets'
    },
  ],
};

/// Descriptor for `WalletList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List walletListDescriptor = $convert.base64Decode(
    'CgpXYWxsZXRMaXN0EjIKB3dhbGxldHMYASADKAsyGC5hcmJpdGVyLmV2bS5XYWxsZXRFbnRyeV'
    'IHd2FsbGV0cw==');

@$core.Deprecated('Use walletCreateResponseDescriptor instead')
const WalletCreateResponse$json = {
  '1': 'WalletCreateResponse',
  '2': [
    {
      '1': 'wallet',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.WalletEntry',
      '9': 0,
      '10': 'wallet'
    },
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `WalletCreateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List walletCreateResponseDescriptor = $convert.base64Decode(
    'ChRXYWxsZXRDcmVhdGVSZXNwb25zZRIyCgZ3YWxsZXQYASABKAsyGC5hcmJpdGVyLmV2bS5XYW'
    'xsZXRFbnRyeUgAUgZ3YWxsZXQSLQoFZXJyb3IYAiABKA4yFS5hcmJpdGVyLmV2bS5Fdm1FcnJv'
    'ckgAUgVlcnJvckIICgZyZXN1bHQ=');

@$core.Deprecated('Use walletListResponseDescriptor instead')
const WalletListResponse$json = {
  '1': 'WalletListResponse',
  '2': [
    {
      '1': 'wallets',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.WalletList',
      '9': 0,
      '10': 'wallets'
    },
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `WalletListResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List walletListResponseDescriptor = $convert.base64Decode(
    'ChJXYWxsZXRMaXN0UmVzcG9uc2USMwoHd2FsbGV0cxgBIAEoCzIXLmFyYml0ZXIuZXZtLldhbG'
    'xldExpc3RIAFIHd2FsbGV0cxItCgVlcnJvchgCIAEoDjIVLmFyYml0ZXIuZXZtLkV2bUVycm9y'
    'SABSBWVycm9yQggKBnJlc3VsdA==');

@$core.Deprecated('Use transactionRateLimitDescriptor instead')
const TransactionRateLimit$json = {
  '1': 'TransactionRateLimit',
  '2': [
    {'1': 'count', '3': 1, '4': 1, '5': 13, '10': 'count'},
    {'1': 'window_secs', '3': 2, '4': 1, '5': 3, '10': 'windowSecs'},
  ],
};

/// Descriptor for `TransactionRateLimit`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List transactionRateLimitDescriptor = $convert.base64Decode(
    'ChRUcmFuc2FjdGlvblJhdGVMaW1pdBIUCgVjb3VudBgBIAEoDVIFY291bnQSHwoLd2luZG93X3'
    'NlY3MYAiABKANSCndpbmRvd1NlY3M=');

@$core.Deprecated('Use volumeRateLimitDescriptor instead')
const VolumeRateLimit$json = {
  '1': 'VolumeRateLimit',
  '2': [
    {'1': 'max_volume', '3': 1, '4': 1, '5': 12, '10': 'maxVolume'},
    {'1': 'window_secs', '3': 2, '4': 1, '5': 3, '10': 'windowSecs'},
  ],
};

/// Descriptor for `VolumeRateLimit`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List volumeRateLimitDescriptor = $convert.base64Decode(
    'Cg9Wb2x1bWVSYXRlTGltaXQSHQoKbWF4X3ZvbHVtZRgBIAEoDFIJbWF4Vm9sdW1lEh8KC3dpbm'
    'Rvd19zZWNzGAIgASgDUgp3aW5kb3dTZWNz');

@$core.Deprecated('Use sharedSettingsDescriptor instead')
const SharedSettings$json = {
  '1': 'SharedSettings',
  '2': [
    {'1': 'wallet_access_id', '3': 1, '4': 1, '5': 5, '10': 'walletAccessId'},
    {'1': 'chain_id', '3': 2, '4': 1, '5': 4, '10': 'chainId'},
    {
      '1': 'valid_from',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '9': 0,
      '10': 'validFrom',
      '17': true
    },
    {
      '1': 'valid_until',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '9': 1,
      '10': 'validUntil',
      '17': true
    },
    {
      '1': 'max_gas_fee_per_gas',
      '3': 5,
      '4': 1,
      '5': 12,
      '9': 2,
      '10': 'maxGasFeePerGas',
      '17': true
    },
    {
      '1': 'max_priority_fee_per_gas',
      '3': 6,
      '4': 1,
      '5': 12,
      '9': 3,
      '10': 'maxPriorityFeePerGas',
      '17': true
    },
    {
      '1': 'rate_limit',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.TransactionRateLimit',
      '9': 4,
      '10': 'rateLimit',
      '17': true
    },
  ],
  '8': [
    {'1': '_valid_from'},
    {'1': '_valid_until'},
    {'1': '_max_gas_fee_per_gas'},
    {'1': '_max_priority_fee_per_gas'},
    {'1': '_rate_limit'},
  ],
};

/// Descriptor for `SharedSettings`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sharedSettingsDescriptor = $convert.base64Decode(
    'Cg5TaGFyZWRTZXR0aW5ncxIoChB3YWxsZXRfYWNjZXNzX2lkGAEgASgFUg53YWxsZXRBY2Nlc3'
    'NJZBIZCghjaGFpbl9pZBgCIAEoBFIHY2hhaW5JZBI+Cgp2YWxpZF9mcm9tGAMgASgLMhouZ29v'
    'Z2xlLnByb3RvYnVmLlRpbWVzdGFtcEgAUgl2YWxpZEZyb22IAQESQAoLdmFsaWRfdW50aWwYBC'
    'ABKAsyGi5nb29nbGUucHJvdG9idWYuVGltZXN0YW1wSAFSCnZhbGlkVW50aWyIAQESMQoTbWF4'
    'X2dhc19mZWVfcGVyX2dhcxgFIAEoDEgCUg9tYXhHYXNGZWVQZXJHYXOIAQESOwoYbWF4X3ByaW'
    '9yaXR5X2ZlZV9wZXJfZ2FzGAYgASgMSANSFG1heFByaW9yaXR5RmVlUGVyR2FziAEBEkUKCnJh'
    'dGVfbGltaXQYByABKAsyIS5hcmJpdGVyLmV2bS5UcmFuc2FjdGlvblJhdGVMaW1pdEgEUglyYX'
    'RlTGltaXSIAQFCDQoLX3ZhbGlkX2Zyb21CDgoMX3ZhbGlkX3VudGlsQhYKFF9tYXhfZ2FzX2Zl'
    'ZV9wZXJfZ2FzQhsKGV9tYXhfcHJpb3JpdHlfZmVlX3Blcl9nYXNCDQoLX3JhdGVfbGltaXQ=');

@$core.Deprecated('Use etherTransferSettingsDescriptor instead')
const EtherTransferSettings$json = {
  '1': 'EtherTransferSettings',
  '2': [
    {'1': 'targets', '3': 1, '4': 3, '5': 12, '10': 'targets'},
    {
      '1': 'limit',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.VolumeRateLimit',
      '10': 'limit'
    },
  ],
};

/// Descriptor for `EtherTransferSettings`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List etherTransferSettingsDescriptor = $convert.base64Decode(
    'ChVFdGhlclRyYW5zZmVyU2V0dGluZ3MSGAoHdGFyZ2V0cxgBIAMoDFIHdGFyZ2V0cxIyCgVsaW'
    '1pdBgCIAEoCzIcLmFyYml0ZXIuZXZtLlZvbHVtZVJhdGVMaW1pdFIFbGltaXQ=');

@$core.Deprecated('Use tokenTransferSettingsDescriptor instead')
const TokenTransferSettings$json = {
  '1': 'TokenTransferSettings',
  '2': [
    {'1': 'token_contract', '3': 1, '4': 1, '5': 12, '10': 'tokenContract'},
    {
      '1': 'target',
      '3': 2,
      '4': 1,
      '5': 12,
      '9': 0,
      '10': 'target',
      '17': true
    },
    {
      '1': 'volume_limits',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.arbiter.evm.VolumeRateLimit',
      '10': 'volumeLimits'
    },
  ],
  '8': [
    {'1': '_target'},
  ],
};

/// Descriptor for `TokenTransferSettings`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenTransferSettingsDescriptor = $convert.base64Decode(
    'ChVUb2tlblRyYW5zZmVyU2V0dGluZ3MSJQoOdG9rZW5fY29udHJhY3QYASABKAxSDXRva2VuQ2'
    '9udHJhY3QSGwoGdGFyZ2V0GAIgASgMSABSBnRhcmdldIgBARJBCg12b2x1bWVfbGltaXRzGAMg'
    'AygLMhwuYXJiaXRlci5ldm0uVm9sdW1lUmF0ZUxpbWl0Ugx2b2x1bWVMaW1pdHNCCQoHX3Rhcm'
    'dldA==');

@$core.Deprecated('Use specificGrantDescriptor instead')
const SpecificGrant$json = {
  '1': 'SpecificGrant',
  '2': [
    {
      '1': 'ether_transfer',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EtherTransferSettings',
      '9': 0,
      '10': 'etherTransfer'
    },
    {
      '1': 'token_transfer',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.TokenTransferSettings',
      '9': 0,
      '10': 'tokenTransfer'
    },
  ],
  '8': [
    {'1': 'grant'},
  ],
};

/// Descriptor for `SpecificGrant`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List specificGrantDescriptor = $convert.base64Decode(
    'Cg1TcGVjaWZpY0dyYW50EksKDmV0aGVyX3RyYW5zZmVyGAEgASgLMiIuYXJiaXRlci5ldm0uRX'
    'RoZXJUcmFuc2ZlclNldHRpbmdzSABSDWV0aGVyVHJhbnNmZXISSwoOdG9rZW5fdHJhbnNmZXIY'
    'AiABKAsyIi5hcmJpdGVyLmV2bS5Ub2tlblRyYW5zZmVyU2V0dGluZ3NIAFINdG9rZW5UcmFuc2'
    'ZlckIHCgVncmFudA==');

@$core.Deprecated('Use etherTransferMeaningDescriptor instead')
const EtherTransferMeaning$json = {
  '1': 'EtherTransferMeaning',
  '2': [
    {'1': 'to', '3': 1, '4': 1, '5': 12, '10': 'to'},
    {'1': 'value', '3': 2, '4': 1, '5': 12, '10': 'value'},
  ],
};

/// Descriptor for `EtherTransferMeaning`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List etherTransferMeaningDescriptor = $convert.base64Decode(
    'ChRFdGhlclRyYW5zZmVyTWVhbmluZxIOCgJ0bxgBIAEoDFICdG8SFAoFdmFsdWUYAiABKAxSBX'
    'ZhbHVl');

@$core.Deprecated('Use tokenInfoDescriptor instead')
const TokenInfo$json = {
  '1': 'TokenInfo',
  '2': [
    {'1': 'symbol', '3': 1, '4': 1, '5': 9, '10': 'symbol'},
    {'1': 'address', '3': 2, '4': 1, '5': 12, '10': 'address'},
    {'1': 'chain_id', '3': 3, '4': 1, '5': 4, '10': 'chainId'},
  ],
};

/// Descriptor for `TokenInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenInfoDescriptor = $convert.base64Decode(
    'CglUb2tlbkluZm8SFgoGc3ltYm9sGAEgASgJUgZzeW1ib2wSGAoHYWRkcmVzcxgCIAEoDFIHYW'
    'RkcmVzcxIZCghjaGFpbl9pZBgDIAEoBFIHY2hhaW5JZA==');

@$core.Deprecated('Use tokenTransferMeaningDescriptor instead')
const TokenTransferMeaning$json = {
  '1': 'TokenTransferMeaning',
  '2': [
    {
      '1': 'token',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.TokenInfo',
      '10': 'token'
    },
    {'1': 'to', '3': 2, '4': 1, '5': 12, '10': 'to'},
    {'1': 'value', '3': 3, '4': 1, '5': 12, '10': 'value'},
  ],
};

/// Descriptor for `TokenTransferMeaning`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenTransferMeaningDescriptor = $convert.base64Decode(
    'ChRUb2tlblRyYW5zZmVyTWVhbmluZxIsCgV0b2tlbhgBIAEoCzIWLmFyYml0ZXIuZXZtLlRva2'
    'VuSW5mb1IFdG9rZW4SDgoCdG8YAiABKAxSAnRvEhQKBXZhbHVlGAMgASgMUgV2YWx1ZQ==');

@$core.Deprecated('Use specificMeaningDescriptor instead')
const SpecificMeaning$json = {
  '1': 'SpecificMeaning',
  '2': [
    {
      '1': 'ether_transfer',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EtherTransferMeaning',
      '9': 0,
      '10': 'etherTransfer'
    },
    {
      '1': 'token_transfer',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.TokenTransferMeaning',
      '9': 0,
      '10': 'tokenTransfer'
    },
  ],
  '8': [
    {'1': 'meaning'},
  ],
};

/// Descriptor for `SpecificMeaning`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List specificMeaningDescriptor = $convert.base64Decode(
    'Cg9TcGVjaWZpY01lYW5pbmcSSgoOZXRoZXJfdHJhbnNmZXIYASABKAsyIS5hcmJpdGVyLmV2bS'
    '5FdGhlclRyYW5zZmVyTWVhbmluZ0gAUg1ldGhlclRyYW5zZmVyEkoKDnRva2VuX3RyYW5zZmVy'
    'GAIgASgLMiEuYXJiaXRlci5ldm0uVG9rZW5UcmFuc2Zlck1lYW5pbmdIAFINdG9rZW5UcmFuc2'
    'ZlckIJCgdtZWFuaW5n');

@$core.Deprecated('Use gasLimitExceededViolationDescriptor instead')
const GasLimitExceededViolation$json = {
  '1': 'GasLimitExceededViolation',
  '2': [
    {
      '1': 'max_gas_fee_per_gas',
      '3': 1,
      '4': 1,
      '5': 12,
      '9': 0,
      '10': 'maxGasFeePerGas',
      '17': true
    },
    {
      '1': 'max_priority_fee_per_gas',
      '3': 2,
      '4': 1,
      '5': 12,
      '9': 1,
      '10': 'maxPriorityFeePerGas',
      '17': true
    },
  ],
  '8': [
    {'1': '_max_gas_fee_per_gas'},
    {'1': '_max_priority_fee_per_gas'},
  ],
};

/// Descriptor for `GasLimitExceededViolation`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List gasLimitExceededViolationDescriptor = $convert.base64Decode(
    'ChlHYXNMaW1pdEV4Y2VlZGVkVmlvbGF0aW9uEjEKE21heF9nYXNfZmVlX3Blcl9nYXMYASABKA'
    'xIAFIPbWF4R2FzRmVlUGVyR2FziAEBEjsKGG1heF9wcmlvcml0eV9mZWVfcGVyX2dhcxgCIAEo'
    'DEgBUhRtYXhQcmlvcml0eUZlZVBlckdhc4gBAUIWChRfbWF4X2dhc19mZWVfcGVyX2dhc0IbCh'
    'lfbWF4X3ByaW9yaXR5X2ZlZV9wZXJfZ2Fz');

@$core.Deprecated('Use evalViolationDescriptor instead')
const EvalViolation$json = {
  '1': 'EvalViolation',
  '2': [
    {
      '1': 'invalid_target',
      '3': 1,
      '4': 1,
      '5': 12,
      '9': 0,
      '10': 'invalidTarget'
    },
    {
      '1': 'gas_limit_exceeded',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.GasLimitExceededViolation',
      '9': 0,
      '10': 'gasLimitExceeded'
    },
    {
      '1': 'rate_limit_exceeded',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'rateLimitExceeded'
    },
    {
      '1': 'volumetric_limit_exceeded',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'volumetricLimitExceeded'
    },
    {
      '1': 'invalid_time',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'invalidTime'
    },
    {
      '1': 'invalid_transaction_type',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'invalidTransactionType'
    },
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `EvalViolation`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evalViolationDescriptor = $convert.base64Decode(
    'Cg1FdmFsVmlvbGF0aW9uEicKDmludmFsaWRfdGFyZ2V0GAEgASgMSABSDWludmFsaWRUYXJnZX'
    'QSVgoSZ2FzX2xpbWl0X2V4Y2VlZGVkGAIgASgLMiYuYXJiaXRlci5ldm0uR2FzTGltaXRFeGNl'
    'ZWRlZFZpb2xhdGlvbkgAUhBnYXNMaW1pdEV4Y2VlZGVkEkgKE3JhdGVfbGltaXRfZXhjZWVkZW'
    'QYAyABKAsyFi5nb29nbGUucHJvdG9idWYuRW1wdHlIAFIRcmF0ZUxpbWl0RXhjZWVkZWQSVAoZ'
    'dm9sdW1ldHJpY19saW1pdF9leGNlZWRlZBgEIAEoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eU'
    'gAUhd2b2x1bWV0cmljTGltaXRFeGNlZWRlZBI7CgxpbnZhbGlkX3RpbWUYBSABKAsyFi5nb29n'
    'bGUucHJvdG9idWYuRW1wdHlIAFILaW52YWxpZFRpbWUSUgoYaW52YWxpZF90cmFuc2FjdGlvbl'
    '90eXBlGAYgASgLMhYuZ29vZ2xlLnByb3RvYnVmLkVtcHR5SABSFmludmFsaWRUcmFuc2FjdGlv'
    'blR5cGVCBgoEa2luZA==');

@$core.Deprecated('Use noMatchingGrantErrorDescriptor instead')
const NoMatchingGrantError$json = {
  '1': 'NoMatchingGrantError',
  '2': [
    {
      '1': 'meaning',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SpecificMeaning',
      '10': 'meaning'
    },
  ],
};

/// Descriptor for `NoMatchingGrantError`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List noMatchingGrantErrorDescriptor = $convert.base64Decode(
    'ChROb01hdGNoaW5nR3JhbnRFcnJvchI2CgdtZWFuaW5nGAEgASgLMhwuYXJiaXRlci5ldm0uU3'
    'BlY2lmaWNNZWFuaW5nUgdtZWFuaW5n');

@$core.Deprecated('Use policyViolationsErrorDescriptor instead')
const PolicyViolationsError$json = {
  '1': 'PolicyViolationsError',
  '2': [
    {
      '1': 'meaning',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SpecificMeaning',
      '10': 'meaning'
    },
    {
      '1': 'violations',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.arbiter.evm.EvalViolation',
      '10': 'violations'
    },
  ],
};

/// Descriptor for `PolicyViolationsError`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List policyViolationsErrorDescriptor = $convert.base64Decode(
    'ChVQb2xpY3lWaW9sYXRpb25zRXJyb3ISNgoHbWVhbmluZxgBIAEoCzIcLmFyYml0ZXIuZXZtLl'
    'NwZWNpZmljTWVhbmluZ1IHbWVhbmluZxI6Cgp2aW9sYXRpb25zGAIgAygLMhouYXJiaXRlci5l'
    'dm0uRXZhbFZpb2xhdGlvblIKdmlvbGF0aW9ucw==');

@$core.Deprecated('Use transactionEvalErrorDescriptor instead')
const TransactionEvalError$json = {
  '1': 'TransactionEvalError',
  '2': [
    {
      '1': 'contract_creation_not_supported',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'contractCreationNotSupported'
    },
    {
      '1': 'unsupported_transaction_type',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'unsupportedTransactionType'
    },
    {
      '1': 'no_matching_grant',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.NoMatchingGrantError',
      '9': 0,
      '10': 'noMatchingGrant'
    },
    {
      '1': 'policy_violations',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.PolicyViolationsError',
      '9': 0,
      '10': 'policyViolations'
    },
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `TransactionEvalError`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List transactionEvalErrorDescriptor = $convert.base64Decode(
    'ChRUcmFuc2FjdGlvbkV2YWxFcnJvchJfCh9jb250cmFjdF9jcmVhdGlvbl9ub3Rfc3VwcG9ydG'
    'VkGAEgASgLMhYuZ29vZ2xlLnByb3RvYnVmLkVtcHR5SABSHGNvbnRyYWN0Q3JlYXRpb25Ob3RT'
    'dXBwb3J0ZWQSWgocdW5zdXBwb3J0ZWRfdHJhbnNhY3Rpb25fdHlwZRgCIAEoCzIWLmdvb2dsZS'
    '5wcm90b2J1Zi5FbXB0eUgAUhp1bnN1cHBvcnRlZFRyYW5zYWN0aW9uVHlwZRJPChFub19tYXRj'
    'aGluZ19ncmFudBgDIAEoCzIhLmFyYml0ZXIuZXZtLk5vTWF0Y2hpbmdHcmFudEVycm9ySABSD2'
    '5vTWF0Y2hpbmdHcmFudBJRChFwb2xpY3lfdmlvbGF0aW9ucxgEIAEoCzIiLmFyYml0ZXIuZXZt'
    'LlBvbGljeVZpb2xhdGlvbnNFcnJvckgAUhBwb2xpY3lWaW9sYXRpb25zQgYKBGtpbmQ=');

@$core.Deprecated('Use evmGrantCreateRequestDescriptor instead')
const EvmGrantCreateRequest$json = {
  '1': 'EvmGrantCreateRequest',
  '2': [
    {
      '1': 'shared',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SharedSettings',
      '10': 'shared'
    },
    {
      '1': 'specific',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SpecificGrant',
      '10': 'specific'
    },
  ],
};

/// Descriptor for `EvmGrantCreateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantCreateRequestDescriptor = $convert.base64Decode(
    'ChVFdm1HcmFudENyZWF0ZVJlcXVlc3QSMwoGc2hhcmVkGAEgASgLMhsuYXJiaXRlci5ldm0uU2'
    'hhcmVkU2V0dGluZ3NSBnNoYXJlZBI2CghzcGVjaWZpYxgCIAEoCzIaLmFyYml0ZXIuZXZtLlNw'
    'ZWNpZmljR3JhbnRSCHNwZWNpZmlj');

@$core.Deprecated('Use evmGrantCreateResponseDescriptor instead')
const EvmGrantCreateResponse$json = {
  '1': 'EvmGrantCreateResponse',
  '2': [
    {'1': 'grant_id', '3': 1, '4': 1, '5': 5, '9': 0, '10': 'grantId'},
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `EvmGrantCreateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantCreateResponseDescriptor = $convert.base64Decode(
    'ChZFdm1HcmFudENyZWF0ZVJlc3BvbnNlEhsKCGdyYW50X2lkGAEgASgFSABSB2dyYW50SWQSLQ'
    'oFZXJyb3IYAiABKA4yFS5hcmJpdGVyLmV2bS5Fdm1FcnJvckgAUgVlcnJvckIICgZyZXN1bHQ=');

@$core.Deprecated('Use evmGrantDeleteRequestDescriptor instead')
const EvmGrantDeleteRequest$json = {
  '1': 'EvmGrantDeleteRequest',
  '2': [
    {'1': 'grant_id', '3': 1, '4': 1, '5': 5, '10': 'grantId'},
  ],
};

/// Descriptor for `EvmGrantDeleteRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantDeleteRequestDescriptor =
    $convert.base64Decode(
        'ChVFdm1HcmFudERlbGV0ZVJlcXVlc3QSGQoIZ3JhbnRfaWQYASABKAVSB2dyYW50SWQ=');

@$core.Deprecated('Use evmGrantDeleteResponseDescriptor instead')
const EvmGrantDeleteResponse$json = {
  '1': 'EvmGrantDeleteResponse',
  '2': [
    {
      '1': 'ok',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'ok'
    },
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `EvmGrantDeleteResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantDeleteResponseDescriptor = $convert.base64Decode(
    'ChZFdm1HcmFudERlbGV0ZVJlc3BvbnNlEigKAm9rGAEgASgLMhYuZ29vZ2xlLnByb3RvYnVmLk'
    'VtcHR5SABSAm9rEi0KBWVycm9yGAIgASgOMhUuYXJiaXRlci5ldm0uRXZtRXJyb3JIAFIFZXJy'
    'b3JCCAoGcmVzdWx0');

@$core.Deprecated('Use grantEntryDescriptor instead')
const GrantEntry$json = {
  '1': 'GrantEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 5, '10': 'id'},
    {'1': 'wallet_access_id', '3': 2, '4': 1, '5': 5, '10': 'walletAccessId'},
    {
      '1': 'shared',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SharedSettings',
      '10': 'shared'
    },
    {
      '1': 'specific',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SpecificGrant',
      '10': 'specific'
    },
  ],
};

/// Descriptor for `GrantEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List grantEntryDescriptor = $convert.base64Decode(
    'CgpHcmFudEVudHJ5Eg4KAmlkGAEgASgFUgJpZBIoChB3YWxsZXRfYWNjZXNzX2lkGAIgASgFUg'
    '53YWxsZXRBY2Nlc3NJZBIzCgZzaGFyZWQYAyABKAsyGy5hcmJpdGVyLmV2bS5TaGFyZWRTZXR0'
    'aW5nc1IGc2hhcmVkEjYKCHNwZWNpZmljGAQgASgLMhouYXJiaXRlci5ldm0uU3BlY2lmaWNHcm'
    'FudFIIc3BlY2lmaWM=');

@$core.Deprecated('Use evmGrantListRequestDescriptor instead')
const EvmGrantListRequest$json = {
  '1': 'EvmGrantListRequest',
  '2': [
    {
      '1': 'wallet_access_id',
      '3': 1,
      '4': 1,
      '5': 5,
      '9': 0,
      '10': 'walletAccessId',
      '17': true
    },
  ],
  '8': [
    {'1': '_wallet_access_id'},
  ],
};

/// Descriptor for `EvmGrantListRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantListRequestDescriptor = $convert.base64Decode(
    'ChNFdm1HcmFudExpc3RSZXF1ZXN0Ei0KEHdhbGxldF9hY2Nlc3NfaWQYASABKAVIAFIOd2FsbG'
    'V0QWNjZXNzSWSIAQFCEwoRX3dhbGxldF9hY2Nlc3NfaWQ=');

@$core.Deprecated('Use evmGrantListResponseDescriptor instead')
const EvmGrantListResponse$json = {
  '1': 'EvmGrantListResponse',
  '2': [
    {
      '1': 'grants',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantList',
      '9': 0,
      '10': 'grants'
    },
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `EvmGrantListResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantListResponseDescriptor = $convert.base64Decode(
    'ChRFdm1HcmFudExpc3RSZXNwb25zZRIzCgZncmFudHMYASABKAsyGS5hcmJpdGVyLmV2bS5Fdm'
    '1HcmFudExpc3RIAFIGZ3JhbnRzEi0KBWVycm9yGAIgASgOMhUuYXJiaXRlci5ldm0uRXZtRXJy'
    'b3JIAFIFZXJyb3JCCAoGcmVzdWx0');

@$core.Deprecated('Use evmGrantListDescriptor instead')
const EvmGrantList$json = {
  '1': 'EvmGrantList',
  '2': [
    {
      '1': 'grants',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.arbiter.evm.GrantEntry',
      '10': 'grants'
    },
  ],
};

/// Descriptor for `EvmGrantList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmGrantListDescriptor = $convert.base64Decode(
    'CgxFdm1HcmFudExpc3QSLwoGZ3JhbnRzGAEgAygLMhcuYXJiaXRlci5ldm0uR3JhbnRFbnRyeV'
    'IGZ3JhbnRz');

@$core.Deprecated('Use evmSignTransactionRequestDescriptor instead')
const EvmSignTransactionRequest$json = {
  '1': 'EvmSignTransactionRequest',
  '2': [
    {'1': 'wallet_address', '3': 1, '4': 1, '5': 12, '10': 'walletAddress'},
    {'1': 'rlp_transaction', '3': 2, '4': 1, '5': 12, '10': 'rlpTransaction'},
  ],
};

/// Descriptor for `EvmSignTransactionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmSignTransactionRequestDescriptor =
    $convert.base64Decode(
        'ChlFdm1TaWduVHJhbnNhY3Rpb25SZXF1ZXN0EiUKDndhbGxldF9hZGRyZXNzGAEgASgMUg13YW'
        'xsZXRBZGRyZXNzEicKD3JscF90cmFuc2FjdGlvbhgCIAEoDFIOcmxwVHJhbnNhY3Rpb24=');

@$core.Deprecated('Use evmSignTransactionResponseDescriptor instead')
const EvmSignTransactionResponse$json = {
  '1': 'EvmSignTransactionResponse',
  '2': [
    {'1': 'signature', '3': 1, '4': 1, '5': 12, '9': 0, '10': 'signature'},
    {
      '1': 'eval_error',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.TransactionEvalError',
      '9': 0,
      '10': 'evalError'
    },
    {
      '1': 'error',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `EvmSignTransactionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmSignTransactionResponseDescriptor = $convert.base64Decode(
    'ChpFdm1TaWduVHJhbnNhY3Rpb25SZXNwb25zZRIeCglzaWduYXR1cmUYASABKAxIAFIJc2lnbm'
    'F0dXJlEkIKCmV2YWxfZXJyb3IYAiABKAsyIS5hcmJpdGVyLmV2bS5UcmFuc2FjdGlvbkV2YWxF'
    'cnJvckgAUglldmFsRXJyb3ISLQoFZXJyb3IYAyABKA4yFS5hcmJpdGVyLmV2bS5Fdm1FcnJvck'
    'gAUgVlcnJvckIICgZyZXN1bHQ=');

@$core.Deprecated('Use evmAnalyzeTransactionRequestDescriptor instead')
const EvmAnalyzeTransactionRequest$json = {
  '1': 'EvmAnalyzeTransactionRequest',
  '2': [
    {'1': 'wallet_address', '3': 1, '4': 1, '5': 12, '10': 'walletAddress'},
    {'1': 'rlp_transaction', '3': 2, '4': 1, '5': 12, '10': 'rlpTransaction'},
  ],
};

/// Descriptor for `EvmAnalyzeTransactionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmAnalyzeTransactionRequestDescriptor =
    $convert.base64Decode(
        'ChxFdm1BbmFseXplVHJhbnNhY3Rpb25SZXF1ZXN0EiUKDndhbGxldF9hZGRyZXNzGAEgASgMUg'
        '13YWxsZXRBZGRyZXNzEicKD3JscF90cmFuc2FjdGlvbhgCIAEoDFIOcmxwVHJhbnNhY3Rpb24=');

@$core.Deprecated('Use evmAnalyzeTransactionResponseDescriptor instead')
const EvmAnalyzeTransactionResponse$json = {
  '1': 'EvmAnalyzeTransactionResponse',
  '2': [
    {
      '1': 'meaning',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.SpecificMeaning',
      '9': 0,
      '10': 'meaning'
    },
    {
      '1': 'eval_error',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.TransactionEvalError',
      '9': 0,
      '10': 'evalError'
    },
    {
      '1': 'error',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.arbiter.evm.EvmError',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `EvmAnalyzeTransactionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evmAnalyzeTransactionResponseDescriptor = $convert.base64Decode(
    'Ch1Fdm1BbmFseXplVHJhbnNhY3Rpb25SZXNwb25zZRI4CgdtZWFuaW5nGAEgASgLMhwuYXJiaX'
    'Rlci5ldm0uU3BlY2lmaWNNZWFuaW5nSABSB21lYW5pbmcSQgoKZXZhbF9lcnJvchgCIAEoCzIh'
    'LmFyYml0ZXIuZXZtLlRyYW5zYWN0aW9uRXZhbEVycm9ySABSCWV2YWxFcnJvchItCgVlcnJvch'
    'gDIAEoDjIVLmFyYml0ZXIuZXZtLkV2bUVycm9ySABSBWVycm9yQggKBnJlc3VsdA==');
