// This is a generated file - do not edit.
//
// Generated from client.proto.

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

@$core.Deprecated('Use clientRequestDescriptor instead')
const ClientRequest$json = {
  '1': 'ClientRequest',
  '2': [
    {'1': 'request_id', '3': 4, '4': 1, '5': 5, '10': 'requestId'},
    {
      '1': 'auth',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.auth.Request',
      '9': 0,
      '10': 'auth'
    },
    {
      '1': 'vault',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.vault.Request',
      '9': 0,
      '10': 'vault'
    },
    {
      '1': 'evm',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.evm.Request',
      '9': 0,
      '10': 'evm'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `ClientRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientRequestDescriptor = $convert.base64Decode(
    'Cg1DbGllbnRSZXF1ZXN0Eh0KCnJlcXVlc3RfaWQYBCABKAVSCXJlcXVlc3RJZBIyCgRhdXRoGA'
    'EgASgLMhwuYXJiaXRlci5jbGllbnQuYXV0aC5SZXF1ZXN0SABSBGF1dGgSNQoFdmF1bHQYAiAB'
    'KAsyHS5hcmJpdGVyLmNsaWVudC52YXVsdC5SZXF1ZXN0SABSBXZhdWx0Ei8KA2V2bRgDIAEoCz'
    'IbLmFyYml0ZXIuY2xpZW50LmV2bS5SZXF1ZXN0SABSA2V2bUIJCgdwYXlsb2Fk');

@$core.Deprecated('Use clientResponseDescriptor instead')
const ClientResponse$json = {
  '1': 'ClientResponse',
  '2': [
    {
      '1': 'request_id',
      '3': 7,
      '4': 1,
      '5': 5,
      '9': 1,
      '10': 'requestId',
      '17': true
    },
    {
      '1': 'auth',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.auth.Response',
      '9': 0,
      '10': 'auth'
    },
    {
      '1': 'vault',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.vault.Response',
      '9': 0,
      '10': 'vault'
    },
    {
      '1': 'evm',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.client.evm.Response',
      '9': 0,
      '10': 'evm'
    },
  ],
  '8': [
    {'1': 'payload'},
    {'1': '_request_id'},
  ],
};

/// Descriptor for `ClientResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientResponseDescriptor = $convert.base64Decode(
    'Cg5DbGllbnRSZXNwb25zZRIiCgpyZXF1ZXN0X2lkGAcgASgFSAFSCXJlcXVlc3RJZIgBARIzCg'
    'RhdXRoGAEgASgLMh0uYXJiaXRlci5jbGllbnQuYXV0aC5SZXNwb25zZUgAUgRhdXRoEjYKBXZh'
    'dWx0GAIgASgLMh4uYXJiaXRlci5jbGllbnQudmF1bHQuUmVzcG9uc2VIAFIFdmF1bHQSMAoDZX'
    'ZtGAMgASgLMhwuYXJiaXRlci5jbGllbnQuZXZtLlJlc3BvbnNlSABSA2V2bUIJCgdwYXlsb2Fk'
    'Qg0KC19yZXF1ZXN0X2lk');
