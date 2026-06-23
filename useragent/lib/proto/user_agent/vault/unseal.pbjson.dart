// This is a generated file - do not edit.
//
// Generated from user_agent/vault/unseal.proto.

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

@$core.Deprecated('Use unsealResultDescriptor instead')
const UnsealResult$json = {
  '1': 'UnsealResult',
  '2': [
    {'1': 'UNSEAL_RESULT_UNSPECIFIED', '2': 0},
    {'1': 'UNSEAL_RESULT_SUCCESS', '2': 1},
    {'1': 'UNSEAL_RESULT_INVALID_KEY', '2': 2},
    {'1': 'UNSEAL_RESULT_UNBOOTSTRAPPED', '2': 3},
  ],
};

/// Descriptor for `UnsealResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List unsealResultDescriptor = $convert.base64Decode(
    'CgxVbnNlYWxSZXN1bHQSHQoZVU5TRUFMX1JFU1VMVF9VTlNQRUNJRklFRBAAEhkKFVVOU0VBTF'
    '9SRVNVTFRfU1VDQ0VTUxABEh0KGVVOU0VBTF9SRVNVTFRfSU5WQUxJRF9LRVkQAhIgChxVTlNF'
    'QUxfUkVTVUxUX1VOQk9PVFNUUkFQUEVEEAM=');

@$core.Deprecated('Use unsealStartDescriptor instead')
const UnsealStart$json = {
  '1': 'UnsealStart',
  '2': [
    {'1': 'client_pubkey', '3': 1, '4': 1, '5': 12, '10': 'clientPubkey'},
  ],
};

/// Descriptor for `UnsealStart`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unsealStartDescriptor = $convert.base64Decode(
    'CgtVbnNlYWxTdGFydBIjCg1jbGllbnRfcHVia2V5GAEgASgMUgxjbGllbnRQdWJrZXk=');

@$core.Deprecated('Use unsealStartResponseDescriptor instead')
const UnsealStartResponse$json = {
  '1': 'UnsealStartResponse',
  '2': [
    {'1': 'server_pubkey', '3': 1, '4': 1, '5': 12, '10': 'serverPubkey'},
  ],
};

/// Descriptor for `UnsealStartResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unsealStartResponseDescriptor = $convert.base64Decode(
    'ChNVbnNlYWxTdGFydFJlc3BvbnNlEiMKDXNlcnZlcl9wdWJrZXkYASABKAxSDHNlcnZlclB1Ym'
    'tleQ==');

@$core.Deprecated('Use unsealEncryptedKeyDescriptor instead')
const UnsealEncryptedKey$json = {
  '1': 'UnsealEncryptedKey',
  '2': [
    {'1': 'nonce', '3': 1, '4': 1, '5': 12, '10': 'nonce'},
    {'1': 'ciphertext', '3': 2, '4': 1, '5': 12, '10': 'ciphertext'},
    {'1': 'associated_data', '3': 3, '4': 1, '5': 12, '10': 'associatedData'},
  ],
};

/// Descriptor for `UnsealEncryptedKey`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unsealEncryptedKeyDescriptor = $convert.base64Decode(
    'ChJVbnNlYWxFbmNyeXB0ZWRLZXkSFAoFbm9uY2UYASABKAxSBW5vbmNlEh4KCmNpcGhlcnRleH'
    'QYAiABKAxSCmNpcGhlcnRleHQSJwoPYXNzb2NpYXRlZF9kYXRhGAMgASgMUg5hc3NvY2lhdGVk'
    'RGF0YQ==');

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'start',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.unseal.UnsealStart',
      '9': 0,
      '10': 'start'
    },
    {
      '1': 'encrypted_key',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.unseal.UnsealEncryptedKey',
      '9': 0,
      '10': 'encryptedKey'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0EkQKBXN0YXJ0GAEgASgLMiwuYXJiaXRlci51c2VyX2FnZW50LnZhdWx0LnVuc2'
    'VhbC5VbnNlYWxTdGFydEgAUgVzdGFydBJaCg1lbmNyeXB0ZWRfa2V5GAIgASgLMjMuYXJiaXRl'
    'ci51c2VyX2FnZW50LnZhdWx0LnVuc2VhbC5VbnNlYWxFbmNyeXB0ZWRLZXlIAFIMZW5jcnlwdG'
    'VkS2V5QgkKB3BheWxvYWQ=');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'start',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.unseal.UnsealStartResponse',
      '9': 0,
      '10': 'start'
    },
    {
      '1': 'result',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.vault.unseal.UnsealResult',
      '9': 0,
      '10': 'result'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRJMCgVzdGFydBgBIAEoCzI0LmFyYml0ZXIudXNlcl9hZ2VudC52YXVsdC51bn'
    'NlYWwuVW5zZWFsU3RhcnRSZXNwb25zZUgAUgVzdGFydBJHCgZyZXN1bHQYAiABKA4yLS5hcmJp'
    'dGVyLnVzZXJfYWdlbnQudmF1bHQudW5zZWFsLlVuc2VhbFJlc3VsdEgAUgZyZXN1bHRCCQoHcG'
    'F5bG9hZA==');
