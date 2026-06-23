// This is a generated file - do not edit.
//
// Generated from client/evm.proto.

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

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'sign_transaction',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmSignTransactionRequest',
      '9': 0,
      '10': 'signTransaction'
    },
    {
      '1': 'analyze_transaction',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmAnalyzeTransactionRequest',
      '9': 0,
      '10': 'analyzeTransaction'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0ElMKEHNpZ25fdHJhbnNhY3Rpb24YASABKAsyJi5hcmJpdGVyLmV2bS5Fdm1TaW'
    'duVHJhbnNhY3Rpb25SZXF1ZXN0SABSD3NpZ25UcmFuc2FjdGlvbhJcChNhbmFseXplX3RyYW5z'
    'YWN0aW9uGAIgASgLMikuYXJiaXRlci5ldm0uRXZtQW5hbHl6ZVRyYW5zYWN0aW9uUmVxdWVzdE'
    'gAUhJhbmFseXplVHJhbnNhY3Rpb25CCQoHcGF5bG9hZA==');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'sign_transaction',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmSignTransactionResponse',
      '9': 0,
      '10': 'signTransaction'
    },
    {
      '1': 'analyze_transaction',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmAnalyzeTransactionResponse',
      '9': 0,
      '10': 'analyzeTransaction'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRJUChBzaWduX3RyYW5zYWN0aW9uGAEgASgLMicuYXJiaXRlci5ldm0uRXZtU2'
    'lnblRyYW5zYWN0aW9uUmVzcG9uc2VIAFIPc2lnblRyYW5zYWN0aW9uEl0KE2FuYWx5emVfdHJh'
    'bnNhY3Rpb24YAiABKAsyKi5hcmJpdGVyLmV2bS5Fdm1BbmFseXplVHJhbnNhY3Rpb25SZXNwb2'
    '5zZUgAUhJhbmFseXplVHJhbnNhY3Rpb25CCQoHcGF5bG9hZA==');
