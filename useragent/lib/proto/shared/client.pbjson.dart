// This is a generated file - do not edit.
//
// Generated from shared/client.proto.

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

@$core.Deprecated('Use clientInfoDescriptor instead')
const ClientInfo$json = {
  '1': 'ClientInfo',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'description',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'version',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'version',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_version'},
  ],
};

/// Descriptor for `ClientInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientInfoDescriptor = $convert.base64Decode(
    'CgpDbGllbnRJbmZvEhIKBG5hbWUYASABKAlSBG5hbWUSJQoLZGVzY3JpcHRpb24YAiABKAlIAF'
    'ILZGVzY3JpcHRpb26IAQESHQoHdmVyc2lvbhgDIAEoCUgBUgd2ZXJzaW9uiAEBQg4KDF9kZXNj'
    'cmlwdGlvbkIKCghfdmVyc2lvbg==');
