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

@$core.Deprecated('Use authResultDescriptor instead')
const AuthResult$json = {
  '1': 'AuthResult',
  '2': [
    {'1': 'AUTH_RESULT_UNSPECIFIED', '2': 0},
    {'1': 'AUTH_RESULT_SUCCESS', '2': 1},
    {'1': 'AUTH_RESULT_INVALID_KEY', '2': 2},
    {'1': 'AUTH_RESULT_INVALID_SIGNATURE', '2': 3},
    {'1': 'AUTH_RESULT_APPROVAL_DENIED', '2': 4},
    {'1': 'AUTH_RESULT_NO_USER_AGENTS_ONLINE', '2': 5},
    {'1': 'AUTH_RESULT_INTERNAL', '2': 6},
  ],
};

/// Descriptor for `AuthResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List authResultDescriptor = $convert.base64Decode(
    'CgpBdXRoUmVzdWx0EhsKF0FVVEhfUkVTVUxUX1VOU1BFQ0lGSUVEEAASFwoTQVVUSF9SRVNVTF'
    'RfU1VDQ0VTUxABEhsKF0FVVEhfUkVTVUxUX0lOVkFMSURfS0VZEAISIQodQVVUSF9SRVNVTFRf'
    'SU5WQUxJRF9TSUdOQVRVUkUQAxIfChtBVVRIX1JFU1VMVF9BUFBST1ZBTF9ERU5JRUQQBBIlCi'
    'FBVVRIX1JFU1VMVF9OT19VU0VSX0FHRU5UU19PTkxJTkUQBRIYChRBVVRIX1JFU1VMVF9JTlRF'
    'Uk5BTBAG');

@$core.Deprecated('Use vaultStateDescriptor instead')
const VaultState$json = {
  '1': 'VaultState',
  '2': [
    {'1': 'VAULT_STATE_UNSPECIFIED', '2': 0},
    {'1': 'VAULT_STATE_UNBOOTSTRAPPED', '2': 1},
    {'1': 'VAULT_STATE_SEALED', '2': 2},
    {'1': 'VAULT_STATE_UNSEALED', '2': 3},
    {'1': 'VAULT_STATE_ERROR', '2': 4},
  ],
};

/// Descriptor for `VaultState`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List vaultStateDescriptor = $convert.base64Decode(
    'CgpWYXVsdFN0YXRlEhsKF1ZBVUxUX1NUQVRFX1VOU1BFQ0lGSUVEEAASHgoaVkFVTFRfU1RBVE'
    'VfVU5CT09UU1RSQVBQRUQQARIWChJWQVVMVF9TVEFURV9TRUFMRUQQAhIYChRWQVVMVF9TVEFU'
    'RV9VTlNFQUxFRBADEhUKEVZBVUxUX1NUQVRFX0VSUk9SEAQ=');

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

@$core.Deprecated('Use clientRequestDescriptor instead')
const ClientRequest$json = {
  '1': 'ClientRequest',
  '2': [
    {'1': 'request_id', '3': 4, '4': 1, '5': 5, '10': 'requestId'},
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
    {
      '1': 'query_vault_state',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'queryVaultState'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `ClientRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientRequestDescriptor = $convert.base64Decode(
    'Cg1DbGllbnRSZXF1ZXN0Eh0KCnJlcXVlc3RfaWQYBCABKAVSCXJlcXVlc3RJZBJcChZhdXRoX2'
    'NoYWxsZW5nZV9yZXF1ZXN0GAEgASgLMiQuYXJiaXRlci5jbGllbnQuQXV0aENoYWxsZW5nZVJl'
    'cXVlc3RIAFIUYXV0aENoYWxsZW5nZVJlcXVlc3QSXwoXYXV0aF9jaGFsbGVuZ2Vfc29sdXRpb2'
    '4YAiABKAsyJS5hcmJpdGVyLmNsaWVudC5BdXRoQ2hhbGxlbmdlU29sdXRpb25IAFIVYXV0aENo'
    'YWxsZW5nZVNvbHV0aW9uEkQKEXF1ZXJ5X3ZhdWx0X3N0YXRlGAMgASgLMhYuZ29vZ2xlLnByb3'
    'RvYnVmLkVtcHR5SABSD3F1ZXJ5VmF1bHRTdGF0ZUIJCgdwYXlsb2Fk');

@$core.Deprecated('Use clientResponseDescriptor instead')
const ClientResponse$json = {
  '1': 'ClientResponse',
  '2': [
    {
      '1': 'request_id',
      '3': 7,
      '4': 1,
      '5': 5,
      '9': 1,
      '10': 'requestId',
      '17': true
    },
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
      '1': 'auth_result',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.client.AuthResult',
      '9': 0,
      '10': 'authResult'
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
    {
      '1': 'vault_state',
      '3': 6,
      '4': 1,
      '5': 14,
      '6': '.arbiter.client.VaultState',
      '9': 0,
      '10': 'vaultState'
    },
  ],
  '8': [
    {'1': 'payload'},
    {'1': '_request_id'},
  ],
};

/// Descriptor for `ClientResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientResponseDescriptor = $convert.base64Decode(
    'Cg5DbGllbnRSZXNwb25zZRIiCgpyZXF1ZXN0X2lkGAcgASgFSAFSCXJlcXVlc3RJZIgBARJGCg'
    '5hdXRoX2NoYWxsZW5nZRgBIAEoCzIdLmFyYml0ZXIuY2xpZW50LkF1dGhDaGFsbGVuZ2VIAFIN'
    'YXV0aENoYWxsZW5nZRI9CgthdXRoX3Jlc3VsdBgCIAEoDjIaLmFyYml0ZXIuY2xpZW50LkF1dG'
    'hSZXN1bHRIAFIKYXV0aFJlc3VsdBJbChRldm1fc2lnbl90cmFuc2FjdGlvbhgDIAEoCzInLmFy'
    'Yml0ZXIuZXZtLkV2bVNpZ25UcmFuc2FjdGlvblJlc3BvbnNlSABSEmV2bVNpZ25UcmFuc2FjdG'
    'lvbhJkChdldm1fYW5hbHl6ZV90cmFuc2FjdGlvbhgEIAEoCzIqLmFyYml0ZXIuZXZtLkV2bUFu'
    'YWx5emVUcmFuc2FjdGlvblJlc3BvbnNlSABSFWV2bUFuYWx5emVUcmFuc2FjdGlvbhI9Cgt2YX'
    'VsdF9zdGF0ZRgGIAEoDjIaLmFyYml0ZXIuY2xpZW50LlZhdWx0U3RhdGVIAFIKdmF1bHRTdGF0'
    'ZUIJCgdwYXlsb2FkQg0KC19yZXF1ZXN0X2lk');
