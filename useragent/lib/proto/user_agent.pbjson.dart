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

@$core.Deprecated('Use keyTypeDescriptor instead')
const KeyType$json = {
  '1': 'KeyType',
  '2': [
    {'1': 'KEY_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'KEY_TYPE_ED25519', '2': 1},
    {'1': 'KEY_TYPE_ECDSA_SECP256K1', '2': 2},
    {'1': 'KEY_TYPE_RSA', '2': 3},
  ],
};

/// Descriptor for `KeyType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List keyTypeDescriptor = $convert.base64Decode(
    'CgdLZXlUeXBlEhgKFEtFWV9UWVBFX1VOU1BFQ0lGSUVEEAASFAoQS0VZX1RZUEVfRUQyNTUxOR'
    'ABEhwKGEtFWV9UWVBFX0VDRFNBX1NFQ1AyNTZLMRACEhAKDEtFWV9UWVBFX1JTQRAD');

@$core.Deprecated('Use authResultDescriptor instead')
const AuthResult$json = {
  '1': 'AuthResult',
  '2': [
    {'1': 'AUTH_RESULT_UNSPECIFIED', '2': 0},
    {'1': 'AUTH_RESULT_SUCCESS', '2': 1},
    {'1': 'AUTH_RESULT_INVALID_KEY', '2': 2},
    {'1': 'AUTH_RESULT_INVALID_SIGNATURE', '2': 3},
    {'1': 'AUTH_RESULT_BOOTSTRAP_REQUIRED', '2': 4},
    {'1': 'AUTH_RESULT_TOKEN_INVALID', '2': 5},
    {'1': 'AUTH_RESULT_INTERNAL', '2': 6},
  ],
};

/// Descriptor for `AuthResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List authResultDescriptor = $convert.base64Decode(
    'CgpBdXRoUmVzdWx0EhsKF0FVVEhfUkVTVUxUX1VOU1BFQ0lGSUVEEAASFwoTQVVUSF9SRVNVTF'
    'RfU1VDQ0VTUxABEhsKF0FVVEhfUkVTVUxUX0lOVkFMSURfS0VZEAISIQodQVVUSF9SRVNVTFRf'
    'SU5WQUxJRF9TSUdOQVRVUkUQAxIiCh5BVVRIX1JFU1VMVF9CT09UU1RSQVBfUkVRVUlSRUQQBB'
    'IdChlBVVRIX1JFU1VMVF9UT0tFTl9JTlZBTElEEAUSGAoUQVVUSF9SRVNVTFRfSU5URVJOQUwQ'
    'Bg==');

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
    {
      '1': 'key_type',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.KeyType',
      '10': 'keyType'
    },
  ],
  '8': [
    {'1': '_bootstrap_token'},
  ],
};

/// Descriptor for `AuthChallengeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeRequestDescriptor = $convert.base64Decode(
    'ChRBdXRoQ2hhbGxlbmdlUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleRIsCg9ib290c3'
    'RyYXBfdG9rZW4YAiABKAlIAFIOYm9vdHN0cmFwVG9rZW6IAQESNgoIa2V5X3R5cGUYAyABKA4y'
    'Gy5hcmJpdGVyLnVzZXJfYWdlbnQuS2V5VHlwZVIHa2V5VHlwZUISChBfYm9vdHN0cmFwX3Rva2'
    'Vu');

@$core.Deprecated('Use authChallengeDescriptor instead')
const AuthChallenge$json = {
  '1': 'AuthChallenge',
  '2': [
    {'1': 'nonce', '3': 2, '4': 1, '5': 5, '10': 'nonce'},
  ],
  '9': [
    {'1': 1, '2': 2},
  ],
};

/// Descriptor for `AuthChallenge`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeDescriptor = $convert.base64Decode(
    'Cg1BdXRoQ2hhbGxlbmdlEhQKBW5vbmNlGAIgASgFUgVub25jZUoECAEQAg==');

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
    {'1': 'id', '3': 14, '4': 1, '5': 5, '10': 'id'},
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
    {
      '1': 'bootstrap_encrypted_key',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.BootstrapEncryptedKey',
      '9': 0,
      '10': 'bootstrapEncryptedKey'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `UserAgentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userAgentRequestDescriptor = $convert.base64Decode(
    'ChBVc2VyQWdlbnRSZXF1ZXN0Eg4KAmlkGA4gASgFUgJpZBJgChZhdXRoX2NoYWxsZW5nZV9yZX'
    'F1ZXN0GAEgASgLMiguYXJiaXRlci51c2VyX2FnZW50LkF1dGhDaGFsbGVuZ2VSZXF1ZXN0SABS'
    'FGF1dGhDaGFsbGVuZ2VSZXF1ZXN0EmMKF2F1dGhfY2hhbGxlbmdlX3NvbHV0aW9uGAIgASgLMi'
    'kuYXJiaXRlci51c2VyX2FnZW50LkF1dGhDaGFsbGVuZ2VTb2x1dGlvbkgAUhVhdXRoQ2hhbGxl'
    'bmdlU29sdXRpb24SRAoMdW5zZWFsX3N0YXJ0GAMgASgLMh8uYXJiaXRlci51c2VyX2FnZW50Ll'
    'Vuc2VhbFN0YXJ0SABSC3Vuc2VhbFN0YXJ0EloKFHVuc2VhbF9lbmNyeXB0ZWRfa2V5GAQgASgL'
    'MiYuYXJiaXRlci51c2VyX2FnZW50LlVuc2VhbEVuY3J5cHRlZEtleUgAUhJ1bnNlYWxFbmNyeX'
    'B0ZWRLZXkSRAoRcXVlcnlfdmF1bHRfc3RhdGUYBSABKAsyFi5nb29nbGUucHJvdG9idWYuRW1w'
    'dHlIAFIPcXVlcnlWYXVsdFN0YXRlEkQKEWV2bV93YWxsZXRfY3JlYXRlGAYgASgLMhYuZ29vZ2'
    'xlLnByb3RvYnVmLkVtcHR5SABSD2V2bVdhbGxldENyZWF0ZRJACg9ldm1fd2FsbGV0X2xpc3QY'
    'ByABKAsyFi5nb29nbGUucHJvdG9idWYuRW1wdHlIAFINZXZtV2FsbGV0TGlzdBJOChBldm1fZ3'
    'JhbnRfY3JlYXRlGAggASgLMiIuYXJiaXRlci5ldm0uRXZtR3JhbnRDcmVhdGVSZXF1ZXN0SABS'
    'DmV2bUdyYW50Q3JlYXRlEk4KEGV2bV9ncmFudF9kZWxldGUYCSABKAsyIi5hcmJpdGVyLmV2bS'
    '5Fdm1HcmFudERlbGV0ZVJlcXVlc3RIAFIOZXZtR3JhbnREZWxldGUSSAoOZXZtX2dyYW50X2xp'
    'c3QYCiABKAsyIC5hcmJpdGVyLmV2bS5Fdm1HcmFudExpc3RSZXF1ZXN0SABSDGV2bUdyYW50TG'
    'lzdBJsChpjbGllbnRfY29ubmVjdGlvbl9yZXNwb25zZRgLIAEoCzIsLmFyYml0ZXIudXNlcl9h'
    'Z2VudC5DbGllbnRDb25uZWN0aW9uUmVzcG9uc2VIAFIYY2xpZW50Q29ubmVjdGlvblJlc3Bvbn'
    'NlEmMKF2Jvb3RzdHJhcF9lbmNyeXB0ZWRfa2V5GAwgASgLMikuYXJiaXRlci51c2VyX2FnZW50'
    'LkJvb3RzdHJhcEVuY3J5cHRlZEtleUgAUhVib290c3RyYXBFbmNyeXB0ZWRLZXlCCQoHcGF5bG'
    '9hZA==');

@$core.Deprecated('Use userAgentResponseDescriptor instead')
const UserAgentResponse$json = {
  '1': 'UserAgentResponse',
  '2': [
    {'1': 'id', '3': 14, '4': 1, '5': 5, '9': 1, '10': 'id', '17': true},
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
      '1': 'auth_result',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.AuthResult',
      '9': 0,
      '10': 'authResult'
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
    {
      '1': 'bootstrap_result',
      '3': 13,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.BootstrapResult',
      '9': 0,
      '10': 'bootstrapResult'
    },
  ],
  '8': [
    {'1': 'payload'},
    {'1': '_id'},
  ],
};

/// Descriptor for `UserAgentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userAgentResponseDescriptor = $convert.base64Decode(
    'ChFVc2VyQWdlbnRSZXNwb25zZRITCgJpZBgOIAEoBUgBUgJpZIgBARJKCg5hdXRoX2NoYWxsZW'
    '5nZRgBIAEoCzIhLmFyYml0ZXIudXNlcl9hZ2VudC5BdXRoQ2hhbGxlbmdlSABSDWF1dGhDaGFs'
    'bGVuZ2USQQoLYXV0aF9yZXN1bHQYAiABKA4yHi5hcmJpdGVyLnVzZXJfYWdlbnQuQXV0aFJlc3'
    'VsdEgAUgphdXRoUmVzdWx0El0KFXVuc2VhbF9zdGFydF9yZXNwb25zZRgDIAEoCzInLmFyYml0'
    'ZXIudXNlcl9hZ2VudC5VbnNlYWxTdGFydFJlc3BvbnNlSABSE3Vuc2VhbFN0YXJ0UmVzcG9uc2'
    'USRwoNdW5zZWFsX3Jlc3VsdBgEIAEoDjIgLmFyYml0ZXIudXNlcl9hZ2VudC5VbnNlYWxSZXN1'
    'bHRIAFIMdW5zZWFsUmVzdWx0EkEKC3ZhdWx0X3N0YXRlGAUgASgOMh4uYXJiaXRlci51c2VyX2'
    'FnZW50LlZhdWx0U3RhdGVIAFIKdmF1bHRTdGF0ZRJPChFldm1fd2FsbGV0X2NyZWF0ZRgGIAEo'
    'CzIhLmFyYml0ZXIuZXZtLldhbGxldENyZWF0ZVJlc3BvbnNlSABSD2V2bVdhbGxldENyZWF0ZR'
    'JJCg9ldm1fd2FsbGV0X2xpc3QYByABKAsyHy5hcmJpdGVyLmV2bS5XYWxsZXRMaXN0UmVzcG9u'
    'c2VIAFINZXZtV2FsbGV0TGlzdBJPChBldm1fZ3JhbnRfY3JlYXRlGAggASgLMiMuYXJiaXRlci'
    '5ldm0uRXZtR3JhbnRDcmVhdGVSZXNwb25zZUgAUg5ldm1HcmFudENyZWF0ZRJPChBldm1fZ3Jh'
    'bnRfZGVsZXRlGAkgASgLMiMuYXJiaXRlci5ldm0uRXZtR3JhbnREZWxldGVSZXNwb25zZUgAUg'
    '5ldm1HcmFudERlbGV0ZRJJCg5ldm1fZ3JhbnRfbGlzdBgKIAEoCzIhLmFyYml0ZXIuZXZtLkV2'
    'bUdyYW50TGlzdFJlc3BvbnNlSABSDGV2bUdyYW50TGlzdBJpChljbGllbnRfY29ubmVjdGlvbl'
    '9yZXF1ZXN0GAsgASgLMisuYXJiaXRlci51c2VyX2FnZW50LkNsaWVudENvbm5lY3Rpb25SZXF1'
    'ZXN0SABSF2NsaWVudENvbm5lY3Rpb25SZXF1ZXN0EmYKGGNsaWVudF9jb25uZWN0aW9uX2Nhbm'
    'NlbBgMIAEoCzIqLmFyYml0ZXIudXNlcl9hZ2VudC5DbGllbnRDb25uZWN0aW9uQ2FuY2VsSABS'
    'FmNsaWVudENvbm5lY3Rpb25DYW5jZWwSUAoQYm9vdHN0cmFwX3Jlc3VsdBgNIAEoDjIjLmFyYm'
    'l0ZXIudXNlcl9hZ2VudC5Cb290c3RyYXBSZXN1bHRIAFIPYm9vdHN0cmFwUmVzdWx0QgkKB3Bh'
    'eWxvYWRCBQoDX2lk');
