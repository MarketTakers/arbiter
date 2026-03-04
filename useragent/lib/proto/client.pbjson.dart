// This is a generated file - do not edit.
//
// Generated from client.proto.

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

@$core.Deprecated('Use authChallengeRequestDescriptor instead')
const AuthChallengeRequest$json = {
  '1': 'AuthChallengeRequest',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
  ],
};

/// Descriptor for `AuthChallengeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeRequestDescriptor =
    $convert.base64Decode(
        'ChRBdXRoQ2hhbGxlbmdlUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleQ==');

@$core.Deprecated('Use authChallengeDescriptor instead')
const AuthChallenge$json = {
  '1': 'AuthChallenge',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
    {'1': 'nonce', '3': 2, '4': 1, '5': 5, '10': 'nonce'},
  ],
};

/// Descriptor for `AuthChallenge`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeDescriptor = $convert.base64Decode(
    'Cg1BdXRoQ2hhbGxlbmdlEhYKBnB1YmtleRgBIAEoDFIGcHVia2V5EhQKBW5vbmNlGAIgASgFUg'
    'Vub25jZQ==');

@$core.Deprecated('Use authChallengeSolutionDescriptor instead')
const AuthChallengeSolution$json = {
  '1': 'AuthChallengeSolution',
  '2': [
    {'1': 'signature', '3': 1, '4': 1, '5': 12, '10': 'signature'},
  ],
};

/// Descriptor for `AuthChallengeSolution`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeSolutionDescriptor = $convert.base64Decode(
    'ChVBdXRoQ2hhbGxlbmdlU29sdXRpb24SHAoJc2lnbmF0dXJlGAEgASgMUglzaWduYXR1cmU=');

@$core.Deprecated('Use authOkDescriptor instead')
const AuthOk$json = {
  '1': 'AuthOk',
};

/// Descriptor for `AuthOk`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authOkDescriptor =
    $convert.base64Decode('CgZBdXRoT2s=');

@$core.Deprecated('Use clientRequestDescriptor instead')
const ClientRequest$json = {
  '1': 'ClientRequest',
  '2': [
    {
      '1': 'auth_challenge_request',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.AuthChallengeRequest',
      '9': 0,
      '10': 'authChallengeRequest'
    },
    {
      '1': 'auth_challenge_solution',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.AuthChallengeSolution',
      '9': 0,
      '10': 'authChallengeSolution'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `ClientRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientRequestDescriptor = $convert.base64Decode(
    'Cg1DbGllbnRSZXF1ZXN0ElwKFmF1dGhfY2hhbGxlbmdlX3JlcXVlc3QYASABKAsyJC5hcmJpdG'
    'VyLmNsaWVudC5BdXRoQ2hhbGxlbmdlUmVxdWVzdEgAUhRhdXRoQ2hhbGxlbmdlUmVxdWVzdBJf'
    'ChdhdXRoX2NoYWxsZW5nZV9zb2x1dGlvbhgCIAEoCzIlLmFyYml0ZXIuY2xpZW50LkF1dGhDaG'
    'FsbGVuZ2VTb2x1dGlvbkgAUhVhdXRoQ2hhbGxlbmdlU29sdXRpb25CCQoHcGF5bG9hZA==');

@$core.Deprecated('Use clientConnectErrorDescriptor instead')
const ClientConnectError$json = {
  '1': 'ClientConnectError',
  '2': [
    {
      '1': 'code',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.arbiter.client.ClientConnectError.Code',
      '10': 'code'
    },
  ],
  '4': [ClientConnectError_Code$json],
};

@$core.Deprecated('Use clientConnectErrorDescriptor instead')
const ClientConnectError_Code$json = {
  '1': 'Code',
  '2': [
    {'1': 'UNKNOWN', '2': 0},
    {'1': 'APPROVAL_DENIED', '2': 1},
    {'1': 'NO_USER_AGENTS_ONLINE', '2': 2},
  ],
};

/// Descriptor for `ClientConnectError`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientConnectErrorDescriptor = $convert.base64Decode(
    'ChJDbGllbnRDb25uZWN0RXJyb3ISOwoEY29kZRgBIAEoDjInLmFyYml0ZXIuY2xpZW50LkNsaW'
    'VudENvbm5lY3RFcnJvci5Db2RlUgRjb2RlIkMKBENvZGUSCwoHVU5LTk9XThAAEhMKD0FQUFJP'
    'VkFMX0RFTklFRBABEhkKFU5PX1VTRVJfQUdFTlRTX09OTElORRAC');

@$core.Deprecated('Use clientResponseDescriptor instead')
const ClientResponse$json = {
  '1': 'ClientResponse',
  '2': [
    {
      '1': 'auth_challenge',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.AuthChallenge',
      '9': 0,
      '10': 'authChallenge'
    },
    {
      '1': 'auth_ok',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.AuthOk',
      '9': 0,
      '10': 'authOk'
    },
    {
      '1': 'client_connect_error',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.ClientConnectError',
      '9': 0,
      '10': 'clientConnectError'
    },
    {
      '1': 'evm_sign_transaction',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmSignTransactionResponse',
      '9': 0,
      '10': 'evmSignTransaction'
    },
    {
      '1': 'evm_analyze_transaction',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmAnalyzeTransactionResponse',
      '9': 0,
      '10': 'evmAnalyzeTransaction'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `ClientResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientResponseDescriptor = $convert.base64Decode(
    'Cg5DbGllbnRSZXNwb25zZRJGCg5hdXRoX2NoYWxsZW5nZRgBIAEoCzIdLmFyYml0ZXIuY2xpZW'
    '50LkF1dGhDaGFsbGVuZ2VIAFINYXV0aENoYWxsZW5nZRIxCgdhdXRoX29rGAIgASgLMhYuYXJi'
    'aXRlci5jbGllbnQuQXV0aE9rSABSBmF1dGhPaxJWChRjbGllbnRfY29ubmVjdF9lcnJvchgFIA'
    'EoCzIiLmFyYml0ZXIuY2xpZW50LkNsaWVudENvbm5lY3RFcnJvckgAUhJjbGllbnRDb25uZWN0'
    'RXJyb3ISWwoUZXZtX3NpZ25fdHJhbnNhY3Rpb24YAyABKAsyJy5hcmJpdGVyLmV2bS5Fdm1TaW'
    'duVHJhbnNhY3Rpb25SZXNwb25zZUgAUhJldm1TaWduVHJhbnNhY3Rpb24SZAoXZXZtX2FuYWx5'
    'emVfdHJhbnNhY3Rpb24YBCABKAsyKi5hcmJpdGVyLmV2bS5Fdm1BbmFseXplVHJhbnNhY3Rpb2'
    '5SZXNwb25zZUgAUhVldm1BbmFseXplVHJhbnNhY3Rpb25CCQoHcGF5bG9hZA==');
