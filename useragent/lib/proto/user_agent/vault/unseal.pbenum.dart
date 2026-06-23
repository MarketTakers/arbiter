// This is a generated file - do not edit.
//
// Generated from user_agent/vault/unseal.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

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

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
