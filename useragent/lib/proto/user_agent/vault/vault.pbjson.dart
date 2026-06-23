// This is a generated file - do not edit.
//
// Generated from user_agent/vault/vault.proto.

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

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'query_state',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'queryState'
    },
    {
      '1': 'unseal',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.unseal.Request',
      '9': 0,
      '10': 'unseal'
    },
    {
      '1': 'bootstrap',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.bootstrap.Request',
      '9': 0,
      '10': 'bootstrap'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0EjkKC3F1ZXJ5X3N0YXRlGAEgASgLMhYuZ29vZ2xlLnByb3RvYnVmLkVtcHR5SA'
    'BSCnF1ZXJ5U3RhdGUSQgoGdW5zZWFsGAIgASgLMiguYXJiaXRlci51c2VyX2FnZW50LnZhdWx0'
    'LnVuc2VhbC5SZXF1ZXN0SABSBnVuc2VhbBJLCglib290c3RyYXAYAyABKAsyKy5hcmJpdGVyLn'
    'VzZXJfYWdlbnQudmF1bHQuYm9vdHN0cmFwLlJlcXVlc3RIAFIJYm9vdHN0cmFwQgkKB3BheWxv'
    'YWQ=');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'state',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.arbiter.shared.VaultState',
      '9': 0,
      '10': 'state'
    },
    {
      '1': 'unseal',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.unseal.Response',
      '9': 0,
      '10': 'unseal'
    },
    {
      '1': 'bootstrap',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.bootstrap.Response',
      '9': 0,
      '10': 'bootstrap'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRIyCgVzdGF0ZRgBIAEoDjIaLmFyYml0ZXIuc2hhcmVkLlZhdWx0U3RhdGVIAF'
    'IFc3RhdGUSQwoGdW5zZWFsGAIgASgLMikuYXJiaXRlci51c2VyX2FnZW50LnZhdWx0LnVuc2Vh'
    'bC5SZXNwb25zZUgAUgZ1bnNlYWwSTAoJYm9vdHN0cmFwGAMgASgLMiwuYXJiaXRlci51c2VyX2'
    'FnZW50LnZhdWx0LmJvb3RzdHJhcC5SZXNwb25zZUgAUglib290c3RyYXBCCQoHcGF5bG9hZA==');
