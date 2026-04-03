// This is a generated file - do not edit.
//
// Generated from user_agent/auth.proto.

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

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
