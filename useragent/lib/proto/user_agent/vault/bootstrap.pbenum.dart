// This is a generated file - do not edit.
//
// Generated from user_agent/vault/bootstrap.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

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

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
