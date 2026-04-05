// This is a generated file - do not edit.
//
// Generated from user_agent.proto.

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

@$core.Deprecated('Use userAgentRequestDescriptor instead')
const UserAgentRequest$json = {
  '1': 'UserAgentRequest',
  '2': [
    {'1': 'id', '3': 16, '4': 1, '5': 5, '10': 'id'},
    {
      '1': 'auth',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.auth.Request',
      '9': 0,
      '10': 'auth'
    },
    {
      '1': 'vault',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.Request',
      '9': 0,
      '10': 'vault'
    },
    {
      '1': 'evm',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.evm.Request',
      '9': 0,
      '10': 'evm'
    },
    {
      '1': 'sdk_client',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.Request',
      '9': 0,
      '10': 'sdkClient'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `UserAgentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userAgentRequestDescriptor = $convert.base64Decode(
    'ChBVc2VyQWdlbnRSZXF1ZXN0Eg4KAmlkGBAgASgFUgJpZBI2CgRhdXRoGAEgASgLMiAuYXJiaX'
    'Rlci51c2VyX2FnZW50LmF1dGguUmVxdWVzdEgAUgRhdXRoEjkKBXZhdWx0GAIgASgLMiEuYXJi'
    'aXRlci51c2VyX2FnZW50LnZhdWx0LlJlcXVlc3RIAFIFdmF1bHQSMwoDZXZtGAMgASgLMh8uYX'
    'JiaXRlci51c2VyX2FnZW50LmV2bS5SZXF1ZXN0SABSA2V2bRJHCgpzZGtfY2xpZW50GAQgASgL'
    'MiYuYXJiaXRlci51c2VyX2FnZW50LnNka19jbGllbnQuUmVxdWVzdEgAUglzZGtDbGllbnRCCQ'
    'oHcGF5bG9hZA==');

@$core.Deprecated('Use userAgentResponseDescriptor instead')
const UserAgentResponse$json = {
  '1': 'UserAgentResponse',
  '2': [
    {'1': 'id', '3': 16, '4': 1, '5': 5, '9': 1, '10': 'id', '17': true},
    {
      '1': 'auth',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.auth.Response',
      '9': 0,
      '10': 'auth'
    },
    {
      '1': 'vault',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.vault.Response',
      '9': 0,
      '10': 'vault'
    },
    {
      '1': 'evm',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.evm.Response',
      '9': 0,
      '10': 'evm'
    },
    {
      '1': 'sdk_client',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.Response',
      '9': 0,
      '10': 'sdkClient'
    },
  ],
  '8': [
    {'1': 'payload'},
    {'1': '_id'},
  ],
};

/// Descriptor for `UserAgentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userAgentResponseDescriptor = $convert.base64Decode(
    'ChFVc2VyQWdlbnRSZXNwb25zZRITCgJpZBgQIAEoBUgBUgJpZIgBARI3CgRhdXRoGAEgASgLMi'
    'EuYXJiaXRlci51c2VyX2FnZW50LmF1dGguUmVzcG9uc2VIAFIEYXV0aBI6CgV2YXVsdBgCIAEo'
    'CzIiLmFyYml0ZXIudXNlcl9hZ2VudC52YXVsdC5SZXNwb25zZUgAUgV2YXVsdBI0CgNldm0YAy'
    'ABKAsyIC5hcmJpdGVyLnVzZXJfYWdlbnQuZXZtLlJlc3BvbnNlSABSA2V2bRJICgpzZGtfY2xp'
    'ZW50GAQgASgLMicuYXJiaXRlci51c2VyX2FnZW50LnNka19jbGllbnQuUmVzcG9uc2VIAFIJc2'
    'RrQ2xpZW50QgkKB3BheWxvYWRCBQoDX2lk');
