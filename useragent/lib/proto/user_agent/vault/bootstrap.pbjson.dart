// This is a generated file - do not edit.
//
// Generated from user_agent/vault/bootstrap.proto.

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

@$core.Deprecated('Use bootstrapResultDescriptor instead')
const BootstrapResult$json = {
  '1': 'BootstrapResult',
  '2': [
    {'1': 'BOOTSTRAP_RESULT_UNSPECIFIED', '2': 0},
    {'1': 'BOOTSTRAP_RESULT_SUCCESS', '2': 1},
    {'1': 'BOOTSTRAP_RESULT_ALREADY_BOOTSTRAPPED', '2': 2},
    {'1': 'BOOTSTRAP_RESULT_INVALID_KEY', '2': 3},
  ],
};

/// Descriptor for `BootstrapResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List bootstrapResultDescriptor = $convert.base64Decode(
    'Cg9Cb290c3RyYXBSZXN1bHQSIAocQk9PVFNUUkFQX1JFU1VMVF9VTlNQRUNJRklFRBAAEhwKGE'
    'JPT1RTVFJBUF9SRVNVTFRfU1VDQ0VTUxABEikKJUJPT1RTVFJBUF9SRVNVTFRfQUxSRUFEWV9C'
    'T09UU1RSQVBQRUQQAhIgChxCT09UU1RSQVBfUkVTVUxUX0lOVkFMSURfS0VZEAM=');

@$core.Deprecated('Use bootstrapEncryptedKeyDescriptor instead')
const BootstrapEncryptedKey$json = {
  '1': 'BootstrapEncryptedKey',
  '2': [
    {'1': 'nonce', '3': 1, '4': 1, '5': 12, '10': 'nonce'},
    {'1': 'ciphertext', '3': 2, '4': 1, '5': 12, '10': 'ciphertext'},
    {'1': 'associated_data', '3': 3, '4': 1, '5': 12, '10': 'associatedData'},
  ],
};

/// Descriptor for `BootstrapEncryptedKey`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bootstrapEncryptedKeyDescriptor = $convert.base64Decode(
    'ChVCb290c3RyYXBFbmNyeXB0ZWRLZXkSFAoFbm9uY2UYASABKAxSBW5vbmNlEh4KCmNpcGhlcn'
    'RleHQYAiABKAxSCmNpcGhlcnRleHQSJwoPYXNzb2NpYXRlZF9kYXRhGAMgASgMUg5hc3NvY2lh'
    'dGVkRGF0YQ==');

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'encrypted_key',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.bootstrap.BootstrapEncryptedKey',
      '10': 'encryptedKey'
    },
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0El4KDWVuY3J5cHRlZF9rZXkYAiABKAsyOS5hcmJpdGVyLnVzZXJfYWdlbnQudm'
    'F1bHQuYm9vdHN0cmFwLkJvb3RzdHJhcEVuY3J5cHRlZEtleVIMZW5jcnlwdGVkS2V5');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'result',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.vault.bootstrap.BootstrapResult',
      '10': 'result'
    },
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRJLCgZyZXN1bHQYASABKA4yMy5hcmJpdGVyLnVzZXJfYWdlbnQudmF1bHQuYm'
    '9vdHN0cmFwLkJvb3RzdHJhcFJlc3VsdFIGcmVzdWx0');
