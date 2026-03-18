// This is a generated file - do not edit.
//
// Generated from client.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class ClientConnectError_Code extends $pb.ProtobufEnum {
  static const ClientConnectError_Code UNKNOWN =
      ClientConnectError_Code._(0, _omitEnumNames ? '' : 'UNKNOWN');
  static const ClientConnectError_Code APPROVAL_DENIED =
      ClientConnectError_Code._(1, _omitEnumNames ? '' : 'APPROVAL_DENIED');
  static const ClientConnectError_Code NO_USER_AGENTS_ONLINE =
      ClientConnectError_Code._(
          2, _omitEnumNames ? '' : 'NO_USER_AGENTS_ONLINE');

  static const $core.List<ClientConnectError_Code> values =
      <ClientConnectError_Code>[
    UNKNOWN,
    APPROVAL_DENIED,
    NO_USER_AGENTS_ONLINE,
  ];

  static final $core.List<ClientConnectError_Code?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 2);
  static ClientConnectError_Code? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ClientConnectError_Code._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
