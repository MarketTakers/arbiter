// This is a generated file - do not edit.
//
// Generated from shared/vault.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class VaultState extends $pb.ProtobufEnum {
  static const VaultState VAULT_STATE_UNSPECIFIED =
      VaultState._(0, _omitEnumNames ? '' : 'VAULT_STATE_UNSPECIFIED');
  static const VaultState VAULT_STATE_UNBOOTSTRAPPED =
      VaultState._(1, _omitEnumNames ? '' : 'VAULT_STATE_UNBOOTSTRAPPED');
  static const VaultState VAULT_STATE_SEALED =
      VaultState._(2, _omitEnumNames ? '' : 'VAULT_STATE_SEALED');
  static const VaultState VAULT_STATE_UNSEALED =
      VaultState._(3, _omitEnumNames ? '' : 'VAULT_STATE_UNSEALED');
  static const VaultState VAULT_STATE_ERROR =
      VaultState._(4, _omitEnumNames ? '' : 'VAULT_STATE_ERROR');

  static const $core.List<VaultState> values = <VaultState>[
    VAULT_STATE_UNSPECIFIED,
    VAULT_STATE_UNBOOTSTRAPPED,
    VAULT_STATE_SEALED,
    VAULT_STATE_UNSEALED,
    VAULT_STATE_ERROR,
  ];

  static final $core.List<VaultState?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static VaultState? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const VaultState._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
