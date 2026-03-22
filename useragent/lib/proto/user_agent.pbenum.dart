// This is a generated file - do not edit.
//
// Generated from user_agent.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class KeyType extends $pb.ProtobufEnum {
  static const KeyType KEY_TYPE_UNSPECIFIED =
      KeyType._(0, _omitEnumNames ? '' : 'KEY_TYPE_UNSPECIFIED');
  static const KeyType KEY_TYPE_ED25519 =
      KeyType._(1, _omitEnumNames ? '' : 'KEY_TYPE_ED25519');
  static const KeyType KEY_TYPE_ECDSA_SECP256K1 =
      KeyType._(2, _omitEnumNames ? '' : 'KEY_TYPE_ECDSA_SECP256K1');
  static const KeyType KEY_TYPE_RSA =
      KeyType._(3, _omitEnumNames ? '' : 'KEY_TYPE_RSA');

  static const $core.List<KeyType> values = <KeyType>[
    KEY_TYPE_UNSPECIFIED,
    KEY_TYPE_ED25519,
    KEY_TYPE_ECDSA_SECP256K1,
    KEY_TYPE_RSA,
  ];

  static final $core.List<KeyType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static KeyType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const KeyType._(super.value, super.name);
}

class SdkClientError extends $pb.ProtobufEnum {
  static const SdkClientError SDK_CLIENT_ERROR_UNSPECIFIED =
      SdkClientError._(0, _omitEnumNames ? '' : 'SDK_CLIENT_ERROR_UNSPECIFIED');
  static const SdkClientError SDK_CLIENT_ERROR_ALREADY_EXISTS =
      SdkClientError._(
          1, _omitEnumNames ? '' : 'SDK_CLIENT_ERROR_ALREADY_EXISTS');
  static const SdkClientError SDK_CLIENT_ERROR_NOT_FOUND =
      SdkClientError._(2, _omitEnumNames ? '' : 'SDK_CLIENT_ERROR_NOT_FOUND');
  static const SdkClientError SDK_CLIENT_ERROR_HAS_RELATED_DATA =
      SdkClientError._(
          3, _omitEnumNames ? '' : 'SDK_CLIENT_ERROR_HAS_RELATED_DATA');
  static const SdkClientError SDK_CLIENT_ERROR_INTERNAL =
      SdkClientError._(4, _omitEnumNames ? '' : 'SDK_CLIENT_ERROR_INTERNAL');

  static const $core.List<SdkClientError> values = <SdkClientError>[
    SDK_CLIENT_ERROR_UNSPECIFIED,
    SDK_CLIENT_ERROR_ALREADY_EXISTS,
    SDK_CLIENT_ERROR_NOT_FOUND,
    SDK_CLIENT_ERROR_HAS_RELATED_DATA,
    SDK_CLIENT_ERROR_INTERNAL,
  ];

  static final $core.List<SdkClientError?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static SdkClientError? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const SdkClientError._(super.value, super.name);
}

class AuthResult extends $pb.ProtobufEnum {
  static const AuthResult AUTH_RESULT_UNSPECIFIED =
      AuthResult._(0, _omitEnumNames ? '' : 'AUTH_RESULT_UNSPECIFIED');
  static const AuthResult AUTH_RESULT_SUCCESS =
      AuthResult._(1, _omitEnumNames ? '' : 'AUTH_RESULT_SUCCESS');
  static const AuthResult AUTH_RESULT_INVALID_KEY =
      AuthResult._(2, _omitEnumNames ? '' : 'AUTH_RESULT_INVALID_KEY');
  static const AuthResult AUTH_RESULT_INVALID_SIGNATURE =
      AuthResult._(3, _omitEnumNames ? '' : 'AUTH_RESULT_INVALID_SIGNATURE');
  static const AuthResult AUTH_RESULT_BOOTSTRAP_REQUIRED =
      AuthResult._(4, _omitEnumNames ? '' : 'AUTH_RESULT_BOOTSTRAP_REQUIRED');
  static const AuthResult AUTH_RESULT_TOKEN_INVALID =
      AuthResult._(5, _omitEnumNames ? '' : 'AUTH_RESULT_TOKEN_INVALID');
  static const AuthResult AUTH_RESULT_INTERNAL =
      AuthResult._(6, _omitEnumNames ? '' : 'AUTH_RESULT_INTERNAL');

  static const $core.List<AuthResult> values = <AuthResult>[
    AUTH_RESULT_UNSPECIFIED,
    AUTH_RESULT_SUCCESS,
    AUTH_RESULT_INVALID_KEY,
    AUTH_RESULT_INVALID_SIGNATURE,
    AUTH_RESULT_BOOTSTRAP_REQUIRED,
    AUTH_RESULT_TOKEN_INVALID,
    AUTH_RESULT_INTERNAL,
  ];

  static final $core.List<AuthResult?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 6);
  static AuthResult? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const AuthResult._(super.value, super.name);
}

class UnsealResult extends $pb.ProtobufEnum {
  static const UnsealResult UNSEAL_RESULT_UNSPECIFIED =
      UnsealResult._(0, _omitEnumNames ? '' : 'UNSEAL_RESULT_UNSPECIFIED');
  static const UnsealResult UNSEAL_RESULT_SUCCESS =
      UnsealResult._(1, _omitEnumNames ? '' : 'UNSEAL_RESULT_SUCCESS');
  static const UnsealResult UNSEAL_RESULT_INVALID_KEY =
      UnsealResult._(2, _omitEnumNames ? '' : 'UNSEAL_RESULT_INVALID_KEY');
  static const UnsealResult UNSEAL_RESULT_UNBOOTSTRAPPED =
      UnsealResult._(3, _omitEnumNames ? '' : 'UNSEAL_RESULT_UNBOOTSTRAPPED');

  static const $core.List<UnsealResult> values = <UnsealResult>[
    UNSEAL_RESULT_UNSPECIFIED,
    UNSEAL_RESULT_SUCCESS,
    UNSEAL_RESULT_INVALID_KEY,
    UNSEAL_RESULT_UNBOOTSTRAPPED,
  ];

  static final $core.List<UnsealResult?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static UnsealResult? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const UnsealResult._(super.value, super.name);
}

class BootstrapResult extends $pb.ProtobufEnum {
  static const BootstrapResult BOOTSTRAP_RESULT_UNSPECIFIED = BootstrapResult._(
      0, _omitEnumNames ? '' : 'BOOTSTRAP_RESULT_UNSPECIFIED');
  static const BootstrapResult BOOTSTRAP_RESULT_SUCCESS =
      BootstrapResult._(1, _omitEnumNames ? '' : 'BOOTSTRAP_RESULT_SUCCESS');
  static const BootstrapResult BOOTSTRAP_RESULT_ALREADY_BOOTSTRAPPED =
      BootstrapResult._(
          2, _omitEnumNames ? '' : 'BOOTSTRAP_RESULT_ALREADY_BOOTSTRAPPED');
  static const BootstrapResult BOOTSTRAP_RESULT_INVALID_KEY = BootstrapResult._(
      3, _omitEnumNames ? '' : 'BOOTSTRAP_RESULT_INVALID_KEY');

  static const $core.List<BootstrapResult> values = <BootstrapResult>[
    BOOTSTRAP_RESULT_UNSPECIFIED,
    BOOTSTRAP_RESULT_SUCCESS,
    BOOTSTRAP_RESULT_ALREADY_BOOTSTRAPPED,
    BOOTSTRAP_RESULT_INVALID_KEY,
  ];

  static final $core.List<BootstrapResult?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static BootstrapResult? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const BootstrapResult._(super.value, super.name);
}

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
