// This is a generated file - do not edit.
//
// Generated from client/auth.proto.

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

@$core.Deprecated('Use authResultDescriptor instead')
const AuthResult$json = {
  '1': 'AuthResult',
  '2': [
    {'1': 'AUTH_RESULT_UNSPECIFIED', '2': 0},
    {'1': 'AUTH_RESULT_SUCCESS', '2': 1},
    {'1': 'AUTH_RESULT_INVALID_KEY', '2': 2},
    {'1': 'AUTH_RESULT_INVALID_SIGNATURE', '2': 3},
    {'1': 'AUTH_RESULT_APPROVAL_DENIED', '2': 4},
    {'1': 'AUTH_RESULT_NO_USER_AGENTS_ONLINE', '2': 5},
    {'1': 'AUTH_RESULT_INTERNAL', '2': 6},
  ],
};

/// Descriptor for `AuthResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List authResultDescriptor = $convert.base64Decode(
    'CgpBdXRoUmVzdWx0EhsKF0FVVEhfUkVTVUxUX1VOU1BFQ0lGSUVEEAASFwoTQVVUSF9SRVNVTF'
    'RfU1VDQ0VTUxABEhsKF0FVVEhfUkVTVUxUX0lOVkFMSURfS0VZEAISIQodQVVUSF9SRVNVTFRf'
    'SU5WQUxJRF9TSUdOQVRVUkUQAxIfChtBVVRIX1JFU1VMVF9BUFBST1ZBTF9ERU5JRUQQBBIlCi'
    'FBVVRIX1JFU1VMVF9OT19VU0VSX0FHRU5UU19PTkxJTkUQBRIYChRBVVRIX1JFU1VMVF9JTlRF'
    'Uk5BTBAG');

@$core.Deprecated('Use authChallengeRequestDescriptor instead')
const AuthChallengeRequest$json = {
  '1': 'AuthChallengeRequest',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
    {
      '1': 'client_info',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.ClientInfo',
      '10': 'clientInfo'
    },
  ],
};

/// Descriptor for `AuthChallengeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeRequestDescriptor = $convert.base64Decode(
    'ChRBdXRoQ2hhbGxlbmdlUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleRI7CgtjbGllbn'
    'RfaW5mbxgCIAEoCzIaLmFyYml0ZXIuc2hhcmVkLkNsaWVudEluZm9SCmNsaWVudEluZm8=');

@$core.Deprecated('Use authChallengeDescriptor instead')
const AuthChallenge$json = {
  '1': 'AuthChallenge',
  '2': [
    {'1': 'timestamp_nanos', '3': 1, '4': 1, '5': 4, '10': 'timestampNanos'},
    {'1': 'random', '3': 2, '4': 1, '5': 12, '10': 'random'},
  ],
};

/// Descriptor for `AuthChallenge`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeDescriptor = $convert.base64Decode(
    'Cg1BdXRoQ2hhbGxlbmdlEicKD3RpbWVzdGFtcF9uYW5vcxgBIAEoBFIOdGltZXN0YW1wTmFub3'
    'MSFgoGcmFuZG9tGAIgASgMUgZyYW5kb20=');

@$core.Deprecated('Use authChallengeSolutionDescriptor instead')
const AuthChallengeSolution$json = {
  '1': 'AuthChallengeSolution',
  '2': [
    {'1': 'signature', '3': 1, '4': 1, '5': 12, '10': 'signature'},
  ],
};

/// Descriptor for `AuthChallengeSolution`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeSolutionDescriptor = $convert.base64Decode(
    'ChVBdXRoQ2hhbGxlbmdlU29sdXRpb24SHAoJc2lnbmF0dXJlGAEgASgMUglzaWduYXR1cmU=');

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'challenge_request',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.auth.AuthChallengeRequest',
      '9': 0,
      '10': 'challengeRequest'
    },
    {
      '1': 'challenge_solution',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.auth.AuthChallengeSolution',
      '9': 0,
      '10': 'challengeSolution'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0ElgKEWNoYWxsZW5nZV9yZXF1ZXN0GAEgASgLMikuYXJiaXRlci5jbGllbnQuYX'
    'V0aC5BdXRoQ2hhbGxlbmdlUmVxdWVzdEgAUhBjaGFsbGVuZ2VSZXF1ZXN0ElsKEmNoYWxsZW5n'
    'ZV9zb2x1dGlvbhgCIAEoCzIqLmFyYml0ZXIuY2xpZW50LmF1dGguQXV0aENoYWxsZW5nZVNvbH'
    'V0aW9uSABSEWNoYWxsZW5nZVNvbHV0aW9uQgkKB3BheWxvYWQ=');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'challenge',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.auth.AuthChallenge',
      '9': 0,
      '10': 'challenge'
    },
    {
      '1': 'result',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.client.auth.AuthResult',
      '9': 0,
      '10': 'result'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRJCCgljaGFsbGVuZ2UYASABKAsyIi5hcmJpdGVyLmNsaWVudC5hdXRoLkF1dG'
    'hDaGFsbGVuZ2VIAFIJY2hhbGxlbmdlEjkKBnJlc3VsdBgCIAEoDjIfLmFyYml0ZXIuY2xpZW50'
    'LmF1dGguQXV0aFJlc3VsdEgAUgZyZXN1bHRCCQoHcGF5bG9hZA==');
