// This is a generated file - do not edit.
//
// Generated from user_agent/evm.proto.

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

@$core.Deprecated('Use signTransactionRequestDescriptor instead')
const SignTransactionRequest$json = {
  '1': 'SignTransactionRequest',
  '2': [
    {'1': 'client_id', '3': 1, '4': 1, '5': 5, '10': 'clientId'},
    {
      '1': 'request',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmSignTransactionRequest',
      '10': 'request'
    },
  ],
};

/// Descriptor for `SignTransactionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List signTransactionRequestDescriptor = $convert.base64Decode(
    'ChZTaWduVHJhbnNhY3Rpb25SZXF1ZXN0EhsKCWNsaWVudF9pZBgBIAEoBVIIY2xpZW50SWQSQA'
    'oHcmVxdWVzdBgCIAEoCzImLmFyYml0ZXIuZXZtLkV2bVNpZ25UcmFuc2FjdGlvblJlcXVlc3RS'
    'B3JlcXVlc3Q=');

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'wallet_create',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'walletCreate'
    },
    {
      '1': 'wallet_list',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'walletList'
    },
    {
      '1': 'grant_create',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantCreateRequest',
      '9': 0,
      '10': 'grantCreate'
    },
    {
      '1': 'grant_delete',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantDeleteRequest',
      '9': 0,
      '10': 'grantDelete'
    },
    {
      '1': 'grant_list',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantListRequest',
      '9': 0,
      '10': 'grantList'
    },
    {
      '1': 'sign_transaction',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.evm.SignTransactionRequest',
      '9': 0,
      '10': 'signTransaction'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0Ej0KDXdhbGxldF9jcmVhdGUYASABKAsyFi5nb29nbGUucHJvdG9idWYuRW1wdH'
    'lIAFIMd2FsbGV0Q3JlYXRlEjkKC3dhbGxldF9saXN0GAIgASgLMhYuZ29vZ2xlLnByb3RvYnVm'
    'LkVtcHR5SABSCndhbGxldExpc3QSRwoMZ3JhbnRfY3JlYXRlGAMgASgLMiIuYXJiaXRlci5ldm'
    '0uRXZtR3JhbnRDcmVhdGVSZXF1ZXN0SABSC2dyYW50Q3JlYXRlEkcKDGdyYW50X2RlbGV0ZRgE'
    'IAEoCzIiLmFyYml0ZXIuZXZtLkV2bUdyYW50RGVsZXRlUmVxdWVzdEgAUgtncmFudERlbGV0ZR'
    'JBCgpncmFudF9saXN0GAUgASgLMiAuYXJiaXRlci5ldm0uRXZtR3JhbnRMaXN0UmVxdWVzdEgA'
    'UglncmFudExpc3QSWwoQc2lnbl90cmFuc2FjdGlvbhgGIAEoCzIuLmFyYml0ZXIudXNlcl9hZ2'
    'VudC5ldm0uU2lnblRyYW5zYWN0aW9uUmVxdWVzdEgAUg9zaWduVHJhbnNhY3Rpb25CCQoHcGF5'
    'bG9hZA==');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'wallet_create',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.WalletCreateResponse',
      '9': 0,
      '10': 'walletCreate'
    },
    {
      '1': 'wallet_list',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.WalletListResponse',
      '9': 0,
      '10': 'walletList'
    },
    {
      '1': 'grant_create',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantCreateResponse',
      '9': 0,
      '10': 'grantCreate'
    },
    {
      '1': 'grant_delete',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantDeleteResponse',
      '9': 0,
      '10': 'grantDelete'
    },
    {
      '1': 'grant_list',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantListResponse',
      '9': 0,
      '10': 'grantList'
    },
    {
      '1': 'sign_transaction',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmSignTransactionResponse',
      '9': 0,
      '10': 'signTransaction'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRJICg13YWxsZXRfY3JlYXRlGAEgASgLMiEuYXJiaXRlci5ldm0uV2FsbGV0Q3'
    'JlYXRlUmVzcG9uc2VIAFIMd2FsbGV0Q3JlYXRlEkIKC3dhbGxldF9saXN0GAIgASgLMh8uYXJi'
    'aXRlci5ldm0uV2FsbGV0TGlzdFJlc3BvbnNlSABSCndhbGxldExpc3QSSAoMZ3JhbnRfY3JlYX'
    'RlGAMgASgLMiMuYXJiaXRlci5ldm0uRXZtR3JhbnRDcmVhdGVSZXNwb25zZUgAUgtncmFudENy'
    'ZWF0ZRJICgxncmFudF9kZWxldGUYBCABKAsyIy5hcmJpdGVyLmV2bS5Fdm1HcmFudERlbGV0ZV'
    'Jlc3BvbnNlSABSC2dyYW50RGVsZXRlEkIKCmdyYW50X2xpc3QYBSABKAsyIS5hcmJpdGVyLmV2'
    'bS5Fdm1HcmFudExpc3RSZXNwb25zZUgAUglncmFudExpc3QSVAoQc2lnbl90cmFuc2FjdGlvbh'
    'gGIAEoCzInLmFyYml0ZXIuZXZtLkV2bVNpZ25UcmFuc2FjdGlvblJlc3BvbnNlSABSD3NpZ25U'
    'cmFuc2FjdGlvbkIJCgdwYXlsb2Fk');
