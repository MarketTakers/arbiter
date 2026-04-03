// This is a generated file - do not edit.
//
// Generated from user_agent/sdk_client.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class Error extends $pb.ProtobufEnum {
  static const Error ERROR_UNSPECIFIED =
      Error._(0, _omitEnumNames ? '' : 'ERROR_UNSPECIFIED');
  static const Error ERROR_ALREADY_EXISTS =
      Error._(1, _omitEnumNames ? '' : 'ERROR_ALREADY_EXISTS');
  static const Error ERROR_NOT_FOUND =
      Error._(2, _omitEnumNames ? '' : 'ERROR_NOT_FOUND');
  static const Error ERROR_HAS_RELATED_DATA =
      Error._(3, _omitEnumNames ? '' : 'ERROR_HAS_RELATED_DATA');
  static const Error ERROR_INTERNAL =
      Error._(4, _omitEnumNames ? '' : 'ERROR_INTERNAL');

  static const $core.List<Error> values = <Error>[
    ERROR_UNSPECIFIED,
    ERROR_ALREADY_EXISTS,
    ERROR_NOT_FOUND,
    ERROR_HAS_RELATED_DATA,
    ERROR_INTERNAL,
  ];

  static final $core.List<Error?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static Error? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const Error._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
