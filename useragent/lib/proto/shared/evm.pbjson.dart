// This is a generated file - do not edit.
//
// Generated from shared/evm.proto.

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
      '6': '.arbiter.shared.evm.TokenInfo',
      '10': 'token'
    },
    {'1': 'to', '3': 2, '4': 1, '5': 12, '10': 'to'},
    {'1': 'value', '3': 3, '4': 1, '5': 12, '10': 'value'},
  ],
};

/// Descriptor for `TokenTransferMeaning`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenTransferMeaningDescriptor = $convert.base64Decode(
    'ChRUb2tlblRyYW5zZmVyTWVhbmluZxIzCgV0b2tlbhgBIAEoCzIdLmFyYml0ZXIuc2hhcmVkLm'
    'V2bS5Ub2tlbkluZm9SBXRva2VuEg4KAnRvGAIgASgMUgJ0bxIUCgV2YWx1ZRgDIAEoDFIFdmFs'
    'dWU=');

@$core.Deprecated('Use specificMeaningDescriptor instead')
const SpecificMeaning$json = {
  '1': 'SpecificMeaning',
  '2': [
    {
      '1': 'ether_transfer',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.evm.EtherTransferMeaning',
      '9': 0,
      '10': 'etherTransfer'
    },
    {
      '1': 'token_transfer',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.evm.TokenTransferMeaning',
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
    'Cg9TcGVjaWZpY01lYW5pbmcSUQoOZXRoZXJfdHJhbnNmZXIYASABKAsyKC5hcmJpdGVyLnNoYX'
    'JlZC5ldm0uRXRoZXJUcmFuc2Zlck1lYW5pbmdIAFINZXRoZXJUcmFuc2ZlchJRCg50b2tlbl90'
    'cmFuc2ZlchgCIAEoCzIoLmFyYml0ZXIuc2hhcmVkLmV2bS5Ub2tlblRyYW5zZmVyTWVhbmluZ0'
    'gAUg10b2tlblRyYW5zZmVyQgkKB21lYW5pbmc=');

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
      '6': '.arbiter.shared.evm.GasLimitExceededViolation',
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
    'QSXQoSZ2FzX2xpbWl0X2V4Y2VlZGVkGAIgASgLMi0uYXJiaXRlci5zaGFyZWQuZXZtLkdhc0xp'
    'bWl0RXhjZWVkZWRWaW9sYXRpb25IAFIQZ2FzTGltaXRFeGNlZWRlZBJIChNyYXRlX2xpbWl0X2'
    'V4Y2VlZGVkGAMgASgLMhYuZ29vZ2xlLnByb3RvYnVmLkVtcHR5SABSEXJhdGVMaW1pdEV4Y2Vl'
    'ZGVkElQKGXZvbHVtZXRyaWNfbGltaXRfZXhjZWVkZWQYBCABKAsyFi5nb29nbGUucHJvdG9idW'
    'YuRW1wdHlIAFIXdm9sdW1ldHJpY0xpbWl0RXhjZWVkZWQSOwoMaW52YWxpZF90aW1lGAUgASgL'
    'MhYuZ29vZ2xlLnByb3RvYnVmLkVtcHR5SABSC2ludmFsaWRUaW1lElIKGGludmFsaWRfdHJhbn'
    'NhY3Rpb25fdHlwZRgGIAEoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eUgAUhZpbnZhbGlkVHJh'
    'bnNhY3Rpb25UeXBlQgYKBGtpbmQ=');

@$core.Deprecated('Use noMatchingGrantErrorDescriptor instead')
const NoMatchingGrantError$json = {
  '1': 'NoMatchingGrantError',
  '2': [
    {
      '1': 'meaning',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.evm.SpecificMeaning',
      '10': 'meaning'
    },
  ],
};

/// Descriptor for `NoMatchingGrantError`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List noMatchingGrantErrorDescriptor = $convert.base64Decode(
    'ChROb01hdGNoaW5nR3JhbnRFcnJvchI9CgdtZWFuaW5nGAEgASgLMiMuYXJiaXRlci5zaGFyZW'
    'QuZXZtLlNwZWNpZmljTWVhbmluZ1IHbWVhbmluZw==');

@$core.Deprecated('Use policyViolationsErrorDescriptor instead')
const PolicyViolationsError$json = {
  '1': 'PolicyViolationsError',
  '2': [
    {
      '1': 'meaning',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.evm.SpecificMeaning',
      '10': 'meaning'
    },
    {
      '1': 'violations',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.arbiter.shared.evm.EvalViolation',
      '10': 'violations'
    },
  ],
};

/// Descriptor for `PolicyViolationsError`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List policyViolationsErrorDescriptor = $convert.base64Decode(
    'ChVQb2xpY3lWaW9sYXRpb25zRXJyb3ISPQoHbWVhbmluZxgBIAEoCzIjLmFyYml0ZXIuc2hhcm'
    'VkLmV2bS5TcGVjaWZpY01lYW5pbmdSB21lYW5pbmcSQQoKdmlvbGF0aW9ucxgCIAMoCzIhLmFy'
    'Yml0ZXIuc2hhcmVkLmV2bS5FdmFsVmlvbGF0aW9uUgp2aW9sYXRpb25z');

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
      '6': '.arbiter.shared.evm.NoMatchingGrantError',
      '9': 0,
      '10': 'noMatchingGrant'
    },
    {
      '1': 'policy_violations',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.evm.PolicyViolationsError',
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
    '5wcm90b2J1Zi5FbXB0eUgAUhp1bnN1cHBvcnRlZFRyYW5zYWN0aW9uVHlwZRJWChFub19tYXRj'
    'aGluZ19ncmFudBgDIAEoCzIoLmFyYml0ZXIuc2hhcmVkLmV2bS5Ob01hdGNoaW5nR3JhbnRFcn'
    'JvckgAUg9ub01hdGNoaW5nR3JhbnQSWAoRcG9saWN5X3Zpb2xhdGlvbnMYBCABKAsyKS5hcmJp'
    'dGVyLnNoYXJlZC5ldm0uUG9saWN5VmlvbGF0aW9uc0Vycm9ySABSEHBvbGljeVZpb2xhdGlvbn'
    'NCBgoEa2luZA==');
