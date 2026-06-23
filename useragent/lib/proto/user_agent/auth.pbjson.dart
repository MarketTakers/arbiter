// This is a generated file - do not edit.
//
// Generated from user_agent/auth.proto.

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
    {'1': 'AUTH_RESULT_BOOTSTRAP_REQUIRED', '2': 4},
    {'1': 'AUTH_RESULT_TOKEN_INVALID', '2': 5},
    {'1': 'AUTH_RESULT_INTERNAL', '2': 6},
  ],
};

/// Descriptor for `AuthResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List authResultDescriptor = $convert.base64Decode(
    'CgpBdXRoUmVzdWx0EhsKF0FVVEhfUkVTVUxUX1VOU1BFQ0lGSUVEEAASFwoTQVVUSF9SRVNVTF'
    'RfU1VDQ0VTUxABEhsKF0FVVEhfUkVTVUxUX0lOVkFMSURfS0VZEAISIQodQVVUSF9SRVNVTFRf'
    'SU5WQUxJRF9TSUdOQVRVUkUQAxIiCh5BVVRIX1JFU1VMVF9CT09UU1RSQVBfUkVRVUlSRUQQBB'
    'IdChlBVVRIX1JFU1VMVF9UT0tFTl9JTlZBTElEEAUSGAoUQVVUSF9SRVNVTFRfSU5URVJOQUwQ'
    'Bg==');

@$core.Deprecated('Use authChallengeRequestDescriptor instead')
const AuthChallengeRequest$json = {
  '1': 'AuthChallengeRequest',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
    {
      '1': 'bootstrap_token',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'bootstrapToken',
      '17': true
    },
  ],
  '8': [
    {'1': '_bootstrap_token'},
  ],
};

/// Descriptor for `AuthChallengeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List authChallengeRequestDescriptor = $convert.base64Decode(
    'ChRBdXRoQ2hhbGxlbmdlUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleRIsCg9ib290c3'
    'RyYXBfdG9rZW4YAiABKAlIAFIOYm9vdHN0cmFwVG9rZW6IAQFCEgoQX2Jvb3RzdHJhcF90b2tl'
    'bg==');

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
      '6': '.arbiter.user_agent.auth.AuthChallengeRequest',
      '9': 0,
      '10': 'challengeRequest'
    },
    {
      '1': 'challenge_solution',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.auth.AuthChallengeSolution',
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
    'CgdSZXF1ZXN0ElwKEWNoYWxsZW5nZV9yZXF1ZXN0GAEgASgLMi0uYXJiaXRlci51c2VyX2FnZW'
    '50LmF1dGguQXV0aENoYWxsZW5nZVJlcXVlc3RIAFIQY2hhbGxlbmdlUmVxdWVzdBJfChJjaGFs'
    'bGVuZ2Vfc29sdXRpb24YAiABKAsyLi5hcmJpdGVyLnVzZXJfYWdlbnQuYXV0aC5BdXRoQ2hhbG'
    'xlbmdlU29sdXRpb25IAFIRY2hhbGxlbmdlU29sdXRpb25CCQoHcGF5bG9hZA==');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'challenge',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.auth.AuthChallenge',
      '9': 0,
      '10': 'challenge'
    },
    {
      '1': 'result',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.auth.AuthResult',
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
    'CghSZXNwb25zZRJGCgljaGFsbGVuZ2UYASABKAsyJi5hcmJpdGVyLnVzZXJfYWdlbnQuYXV0aC'
    '5BdXRoQ2hhbGxlbmdlSABSCWNoYWxsZW5nZRI9CgZyZXN1bHQYAiABKA4yIy5hcmJpdGVyLnVz'
    'ZXJfYWdlbnQuYXV0aC5BdXRoUmVzdWx0SABSBnJlc3VsdEIJCgdwYXlsb2Fk');
