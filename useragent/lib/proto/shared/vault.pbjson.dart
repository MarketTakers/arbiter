// This is a generated file - do not edit.
//
// Generated from shared/vault.proto.

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
