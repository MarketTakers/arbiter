// This is a generated file - do not edit.
//
// Generated from user_agent.proto.

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
    {
      '1': 'bootstrap_token',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'bootstrapToken',
      '17': true
    },
  ],
  '8': [
    {'1': '_bootstrap_token'},
  ],
};

/// Descriptor for `AuthChallengeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeRequestDescriptor = $convert.base64Decode(
    'ChRBdXRoQ2hhbGxlbmdlUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleRIsCg9ib290c3'
    'RyYXBfdG9rZW4YAiABKAlIAFIOYm9vdHN0cmFwVG9rZW6IAQFCEgoQX2Jvb3RzdHJhcF90b2tl'
    'bg==');

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

@$core.Deprecated('Use clientConnectionRequestDescriptor instead')
const ClientConnectionRequest$json = {
  '1': 'ClientConnectionRequest',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
  ],
};

/// Descriptor for `ClientConnectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientConnectionRequestDescriptor =
    $convert.base64Decode(
        'ChdDbGllbnRDb25uZWN0aW9uUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleQ==');

@$core.Deprecated('Use clientConnectionResponseDescriptor instead')
const ClientConnectionResponse$json = {
  '1': 'ClientConnectionResponse',
  '2': [
    {'1': 'approved', '3': 1, '4': 1, '5': 8, '10': 'approved'},
  ],
};

/// Descriptor for `ClientConnectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientConnectionResponseDescriptor =
    $convert.base64Decode(
        'ChhDbGllbnRDb25uZWN0aW9uUmVzcG9uc2USGgoIYXBwcm92ZWQYASABKAhSCGFwcHJvdmVk');

@$core.Deprecated('Use clientConnectionCancelDescriptor instead')
const ClientConnectionCancel$json = {
  '1': 'ClientConnectionCancel',
};

/// Descriptor for `ClientConnectionCancel`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientConnectionCancelDescriptor =
    $convert.base64Decode('ChZDbGllbnRDb25uZWN0aW9uQ2FuY2Vs');

@$core.Deprecated('Use userAgentRequestDescriptor instead')
const UserAgentRequest$json = {
  '1': 'UserAgentRequest',
  '2': [
    {
      '1': 'auth_challenge_request',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.AuthChallengeRequest',
      '9': 0,
      '10': 'authChallengeRequest'
    },
    {
      '1': 'auth_challenge_solution',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.AuthChallengeSolution',
      '9': 0,
      '10': 'authChallengeSolution'
    },
    {
      '1': 'unseal_start',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.UnsealStart',
      '9': 0,
      '10': 'unsealStart'
    },
    {
      '1': 'unseal_encrypted_key',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.UnsealEncryptedKey',
      '9': 0,
      '10': 'unsealEncryptedKey'
    },
    {
      '1': 'query_vault_state',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'queryVaultState'
    },
    {
      '1': 'evm_wallet_create',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'evmWalletCreate'
    },
    {
      '1': 'evm_wallet_list',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'evmWalletList'
    },
    {
      '1': 'evm_grant_create',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantCreateRequest',
      '9': 0,
      '10': 'evmGrantCreate'
    },
    {
      '1': 'evm_grant_delete',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantDeleteRequest',
      '9': 0,
      '10': 'evmGrantDelete'
    },
    {
      '1': 'evm_grant_list',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantListRequest',
      '9': 0,
      '10': 'evmGrantList'
    },
    {
      '1': 'client_connection_response',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.ClientConnectionResponse',
      '9': 0,
      '10': 'clientConnectionResponse'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `UserAgentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userAgentRequestDescriptor = $convert.base64Decode(
    'ChBVc2VyQWdlbnRSZXF1ZXN0EmAKFmF1dGhfY2hhbGxlbmdlX3JlcXVlc3QYASABKAsyKC5hcm'
    'JpdGVyLnVzZXJfYWdlbnQuQXV0aENoYWxsZW5nZVJlcXVlc3RIAFIUYXV0aENoYWxsZW5nZVJl'
    'cXVlc3QSYwoXYXV0aF9jaGFsbGVuZ2Vfc29sdXRpb24YAiABKAsyKS5hcmJpdGVyLnVzZXJfYW'
    'dlbnQuQXV0aENoYWxsZW5nZVNvbHV0aW9uSABSFWF1dGhDaGFsbGVuZ2VTb2x1dGlvbhJECgx1'
    'bnNlYWxfc3RhcnQYAyABKAsyHy5hcmJpdGVyLnVzZXJfYWdlbnQuVW5zZWFsU3RhcnRIAFILdW'
    '5zZWFsU3RhcnQSWgoUdW5zZWFsX2VuY3J5cHRlZF9rZXkYBCABKAsyJi5hcmJpdGVyLnVzZXJf'
    'YWdlbnQuVW5zZWFsRW5jcnlwdGVkS2V5SABSEnVuc2VhbEVuY3J5cHRlZEtleRJEChFxdWVyeV'
    '92YXVsdF9zdGF0ZRgFIAEoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eUgAUg9xdWVyeVZhdWx0'
    'U3RhdGUSRAoRZXZtX3dhbGxldF9jcmVhdGUYBiABKAsyFi5nb29nbGUucHJvdG9idWYuRW1wdH'
    'lIAFIPZXZtV2FsbGV0Q3JlYXRlEkAKD2V2bV93YWxsZXRfbGlzdBgHIAEoCzIWLmdvb2dsZS5w'
    'cm90b2J1Zi5FbXB0eUgAUg1ldm1XYWxsZXRMaXN0Ek4KEGV2bV9ncmFudF9jcmVhdGUYCCABKA'
    'syIi5hcmJpdGVyLmV2bS5Fdm1HcmFudENyZWF0ZVJlcXVlc3RIAFIOZXZtR3JhbnRDcmVhdGUS'
    'TgoQZXZtX2dyYW50X2RlbGV0ZRgJIAEoCzIiLmFyYml0ZXIuZXZtLkV2bUdyYW50RGVsZXRlUm'
    'VxdWVzdEgAUg5ldm1HcmFudERlbGV0ZRJICg5ldm1fZ3JhbnRfbGlzdBgKIAEoCzIgLmFyYml0'
    'ZXIuZXZtLkV2bUdyYW50TGlzdFJlcXVlc3RIAFIMZXZtR3JhbnRMaXN0EmwKGmNsaWVudF9jb2'
    '5uZWN0aW9uX3Jlc3BvbnNlGAsgASgLMiwuYXJiaXRlci51c2VyX2FnZW50LkNsaWVudENvbm5l'
    'Y3Rpb25SZXNwb25zZUgAUhhjbGllbnRDb25uZWN0aW9uUmVzcG9uc2VCCQoHcGF5bG9hZA==');

@$core.Deprecated('Use userAgentResponseDescriptor instead')
const UserAgentResponse$json = {
  '1': 'UserAgentResponse',
  '2': [
    {
      '1': 'auth_challenge',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.AuthChallenge',
      '9': 0,
      '10': 'authChallenge'
    },
    {
      '1': 'auth_ok',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.AuthOk',
      '9': 0,
      '10': 'authOk'
    },
    {
      '1': 'unseal_start_response',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.UnsealStartResponse',
      '9': 0,
      '10': 'unsealStartResponse'
    },
    {
      '1': 'unseal_result',
      '3': 4,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.UnsealResult',
      '9': 0,
      '10': 'unsealResult'
    },
    {
      '1': 'vault_state',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.VaultState',
      '9': 0,
      '10': 'vaultState'
    },
    {
      '1': 'evm_wallet_create',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.WalletCreateResponse',
      '9': 0,
      '10': 'evmWalletCreate'
    },
    {
      '1': 'evm_wallet_list',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.WalletListResponse',
      '9': 0,
      '10': 'evmWalletList'
    },
    {
      '1': 'evm_grant_create',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantCreateResponse',
      '9': 0,
      '10': 'evmGrantCreate'
    },
    {
      '1': 'evm_grant_delete',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantDeleteResponse',
      '9': 0,
      '10': 'evmGrantDelete'
    },
    {
      '1': 'evm_grant_list',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.arbiter.evm.EvmGrantListResponse',
      '9': 0,
      '10': 'evmGrantList'
    },
    {
      '1': 'client_connection_request',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.ClientConnectionRequest',
      '9': 0,
      '10': 'clientConnectionRequest'
    },
    {
      '1': 'client_connection_cancel',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.ClientConnectionCancel',
      '9': 0,
      '10': 'clientConnectionCancel'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `UserAgentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userAgentResponseDescriptor = $convert.base64Decode(
    'ChFVc2VyQWdlbnRSZXNwb25zZRJKCg5hdXRoX2NoYWxsZW5nZRgBIAEoCzIhLmFyYml0ZXIudX'
    'Nlcl9hZ2VudC5BdXRoQ2hhbGxlbmdlSABSDWF1dGhDaGFsbGVuZ2USNQoHYXV0aF9vaxgCIAEo'
    'CzIaLmFyYml0ZXIudXNlcl9hZ2VudC5BdXRoT2tIAFIGYXV0aE9rEl0KFXVuc2VhbF9zdGFydF'
    '9yZXNwb25zZRgDIAEoCzInLmFyYml0ZXIudXNlcl9hZ2VudC5VbnNlYWxTdGFydFJlc3BvbnNl'
    'SABSE3Vuc2VhbFN0YXJ0UmVzcG9uc2USRwoNdW5zZWFsX3Jlc3VsdBgEIAEoDjIgLmFyYml0ZX'
    'IudXNlcl9hZ2VudC5VbnNlYWxSZXN1bHRIAFIMdW5zZWFsUmVzdWx0EkEKC3ZhdWx0X3N0YXRl'
    'GAUgASgOMh4uYXJiaXRlci51c2VyX2FnZW50LlZhdWx0U3RhdGVIAFIKdmF1bHRTdGF0ZRJPCh'
    'Fldm1fd2FsbGV0X2NyZWF0ZRgGIAEoCzIhLmFyYml0ZXIuZXZtLldhbGxldENyZWF0ZVJlc3Bv'
    'bnNlSABSD2V2bVdhbGxldENyZWF0ZRJJCg9ldm1fd2FsbGV0X2xpc3QYByABKAsyHy5hcmJpdG'
    'VyLmV2bS5XYWxsZXRMaXN0UmVzcG9uc2VIAFINZXZtV2FsbGV0TGlzdBJPChBldm1fZ3JhbnRf'
    'Y3JlYXRlGAggASgLMiMuYXJiaXRlci5ldm0uRXZtR3JhbnRDcmVhdGVSZXNwb25zZUgAUg5ldm'
    '1HcmFudENyZWF0ZRJPChBldm1fZ3JhbnRfZGVsZXRlGAkgASgLMiMuYXJiaXRlci5ldm0uRXZt'
    'R3JhbnREZWxldGVSZXNwb25zZUgAUg5ldm1HcmFudERlbGV0ZRJJCg5ldm1fZ3JhbnRfbGlzdB'
    'gKIAEoCzIhLmFyYml0ZXIuZXZtLkV2bUdyYW50TGlzdFJlc3BvbnNlSABSDGV2bUdyYW50TGlz'
    'dBJpChljbGllbnRfY29ubmVjdGlvbl9yZXF1ZXN0GAsgASgLMisuYXJiaXRlci51c2VyX2FnZW'
    '50LkNsaWVudENvbm5lY3Rpb25SZXF1ZXN0SABSF2NsaWVudENvbm5lY3Rpb25SZXF1ZXN0EmYK'
    'GGNsaWVudF9jb25uZWN0aW9uX2NhbmNlbBgMIAEoCzIqLmFyYml0ZXIudXNlcl9hZ2VudC5DbG'
    'llbnRDb25uZWN0aW9uQ2FuY2VsSABSFmNsaWVudENvbm5lY3Rpb25DYW5jZWxCCQoHcGF5bG9h'
    'ZA==');
