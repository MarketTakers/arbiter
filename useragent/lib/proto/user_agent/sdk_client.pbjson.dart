// This is a generated file - do not edit.
//
// Generated from user_agent/sdk_client.proto.

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

@$core.Deprecated('Use errorDescriptor instead')
const Error$json = {
  '1': 'Error',
  '2': [
    {'1': 'ERROR_UNSPECIFIED', '2': 0},
    {'1': 'ERROR_ALREADY_EXISTS', '2': 1},
    {'1': 'ERROR_NOT_FOUND', '2': 2},
    {'1': 'ERROR_HAS_RELATED_DATA', '2': 3},
    {'1': 'ERROR_INTERNAL', '2': 4},
  ],
};

/// Descriptor for `Error`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List errorDescriptor = $convert.base64Decode(
    'CgVFcnJvchIVChFFUlJPUl9VTlNQRUNJRklFRBAAEhgKFEVSUk9SX0FMUkVBRFlfRVhJU1RTEA'
    'ESEwoPRVJST1JfTk9UX0ZPVU5EEAISGgoWRVJST1JfSEFTX1JFTEFURURfREFUQRADEhIKDkVS'
    'Uk9SX0lOVEVSTkFMEAQ=');

@$core.Deprecated('Use revokeRequestDescriptor instead')
const RevokeRequest$json = {
  '1': 'RevokeRequest',
  '2': [
    {'1': 'client_id', '3': 1, '4': 1, '5': 5, '10': 'clientId'},
  ],
};

/// Descriptor for `RevokeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeRequestDescriptor = $convert.base64Decode(
    'Cg1SZXZva2VSZXF1ZXN0EhsKCWNsaWVudF9pZBgBIAEoBVIIY2xpZW50SWQ=');

@$core.Deprecated('Use entryDescriptor instead')
const Entry$json = {
  '1': 'Entry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 5, '10': 'id'},
    {'1': 'pubkey', '3': 2, '4': 1, '5': 12, '10': 'pubkey'},
    {
      '1': 'info',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.ClientInfo',
      '10': 'info'
    },
    {'1': 'created_at', '3': 4, '4': 1, '5': 5, '10': 'createdAt'},
  ],
};

/// Descriptor for `Entry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List entryDescriptor = $convert.base64Decode(
    'CgVFbnRyeRIOCgJpZBgBIAEoBVICaWQSFgoGcHVia2V5GAIgASgMUgZwdWJrZXkSLgoEaW5mbx'
    'gDIAEoCzIaLmFyYml0ZXIuc2hhcmVkLkNsaWVudEluZm9SBGluZm8SHQoKY3JlYXRlZF9hdBgE'
    'IAEoBVIJY3JlYXRlZEF0');

@$core.Deprecated('Use list_Descriptor instead')
const List_$json = {
  '1': 'List',
  '2': [
    {
      '1': 'clients',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.Entry',
      '10': 'clients'
    },
  ],
};

/// Descriptor for `List`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List list_Descriptor = $convert.base64Decode(
    'CgRMaXN0Ej4KB2NsaWVudHMYASADKAsyJC5hcmJpdGVyLnVzZXJfYWdlbnQuc2RrX2NsaWVudC'
    '5FbnRyeVIHY2xpZW50cw==');

@$core.Deprecated('Use revokeResponseDescriptor instead')
const RevokeResponse$json = {
  '1': 'RevokeResponse',
  '2': [
    {
      '1': 'ok',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'ok'
    },
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.sdk_client.Error',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `RevokeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeResponseDescriptor = $convert.base64Decode(
    'Cg5SZXZva2VSZXNwb25zZRIoCgJvaxgBIAEoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eUgAUg'
    'JvaxI8CgVlcnJvchgCIAEoDjIkLmFyYml0ZXIudXNlcl9hZ2VudC5zZGtfY2xpZW50LkVycm9y'
    'SABSBWVycm9yQggKBnJlc3VsdA==');

@$core.Deprecated('Use listResponseDescriptor instead')
const ListResponse$json = {
  '1': 'ListResponse',
  '2': [
    {
      '1': 'clients',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.List',
      '9': 0,
      '10': 'clients'
    },
    {
      '1': 'error',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.arbiter.user_agent.sdk_client.Error',
      '9': 0,
      '10': 'error'
    },
  ],
  '8': [
    {'1': 'result'},
  ],
};

/// Descriptor for `ListResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listResponseDescriptor = $convert.base64Decode(
    'CgxMaXN0UmVzcG9uc2USPwoHY2xpZW50cxgBIAEoCzIjLmFyYml0ZXIudXNlcl9hZ2VudC5zZG'
    'tfY2xpZW50Lkxpc3RIAFIHY2xpZW50cxI8CgVlcnJvchgCIAEoDjIkLmFyYml0ZXIudXNlcl9h'
    'Z2VudC5zZGtfY2xpZW50LkVycm9ySABSBWVycm9yQggKBnJlc3VsdA==');

@$core.Deprecated('Use connectionRequestDescriptor instead')
const ConnectionRequest$json = {
  '1': 'ConnectionRequest',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
    {
      '1': 'info',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.shared.ClientInfo',
      '10': 'info'
    },
  ],
};

/// Descriptor for `ConnectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectionRequestDescriptor = $convert.base64Decode(
    'ChFDb25uZWN0aW9uUmVxdWVzdBIWCgZwdWJrZXkYASABKAxSBnB1YmtleRIuCgRpbmZvGAIgAS'
    'gLMhouYXJiaXRlci5zaGFyZWQuQ2xpZW50SW5mb1IEaW5mbw==');

@$core.Deprecated('Use connectionResponseDescriptor instead')
const ConnectionResponse$json = {
  '1': 'ConnectionResponse',
  '2': [
    {'1': 'approved', '3': 1, '4': 1, '5': 8, '10': 'approved'},
    {'1': 'pubkey', '3': 2, '4': 1, '5': 12, '10': 'pubkey'},
  ],
};

/// Descriptor for `ConnectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectionResponseDescriptor = $convert.base64Decode(
    'ChJDb25uZWN0aW9uUmVzcG9uc2USGgoIYXBwcm92ZWQYASABKAhSCGFwcHJvdmVkEhYKBnB1Ym'
    'tleRgCIAEoDFIGcHVia2V5');

@$core.Deprecated('Use connectionCancelDescriptor instead')
const ConnectionCancel$json = {
  '1': 'ConnectionCancel',
  '2': [
    {'1': 'pubkey', '3': 1, '4': 1, '5': 12, '10': 'pubkey'},
  ],
};

/// Descriptor for `ConnectionCancel`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectionCancelDescriptor = $convert
    .base64Decode('ChBDb25uZWN0aW9uQ2FuY2VsEhYKBnB1YmtleRgBIAEoDFIGcHVia2V5');

@$core.Deprecated('Use walletAccessDescriptor instead')
const WalletAccess$json = {
  '1': 'WalletAccess',
  '2': [
    {'1': 'wallet_id', '3': 1, '4': 1, '5': 5, '10': 'walletId'},
    {'1': 'sdk_client_id', '3': 2, '4': 1, '5': 5, '10': 'sdkClientId'},
  ],
};

/// Descriptor for `WalletAccess`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List walletAccessDescriptor = $convert.base64Decode(
    'CgxXYWxsZXRBY2Nlc3MSGwoJd2FsbGV0X2lkGAEgASgFUgh3YWxsZXRJZBIiCg1zZGtfY2xpZW'
    '50X2lkGAIgASgFUgtzZGtDbGllbnRJZA==');

@$core.Deprecated('Use walletAccessEntryDescriptor instead')
const WalletAccessEntry$json = {
  '1': 'WalletAccessEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 5, '10': 'id'},
    {
      '1': 'access',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.WalletAccess',
      '10': 'access'
    },
  ],
};

/// Descriptor for `WalletAccessEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List walletAccessEntryDescriptor = $convert.base64Decode(
    'ChFXYWxsZXRBY2Nlc3NFbnRyeRIOCgJpZBgBIAEoBVICaWQSQwoGYWNjZXNzGAIgASgLMisuYX'
    'JiaXRlci51c2VyX2FnZW50LnNka19jbGllbnQuV2FsbGV0QWNjZXNzUgZhY2Nlc3M=');

@$core.Deprecated('Use grantWalletAccessDescriptor instead')
const GrantWalletAccess$json = {
  '1': 'GrantWalletAccess',
  '2': [
    {
      '1': 'accesses',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.WalletAccess',
      '10': 'accesses'
    },
  ],
};

/// Descriptor for `GrantWalletAccess`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List grantWalletAccessDescriptor = $convert.base64Decode(
    'ChFHcmFudFdhbGxldEFjY2VzcxJHCghhY2Nlc3NlcxgBIAMoCzIrLmFyYml0ZXIudXNlcl9hZ2'
    'VudC5zZGtfY2xpZW50LldhbGxldEFjY2Vzc1IIYWNjZXNzZXM=');

@$core.Deprecated('Use revokeWalletAccessDescriptor instead')
const RevokeWalletAccess$json = {
  '1': 'RevokeWalletAccess',
  '2': [
    {'1': 'accesses', '3': 1, '4': 3, '5': 5, '10': 'accesses'},
  ],
};

/// Descriptor for `RevokeWalletAccess`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeWalletAccessDescriptor =
    $convert.base64Decode(
        'ChJSZXZva2VXYWxsZXRBY2Nlc3MSGgoIYWNjZXNzZXMYASADKAVSCGFjY2Vzc2Vz');

@$core.Deprecated('Use listWalletAccessResponseDescriptor instead')
const ListWalletAccessResponse$json = {
  '1': 'ListWalletAccessResponse',
  '2': [
    {
      '1': 'accesses',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.WalletAccessEntry',
      '10': 'accesses'
    },
  ],
};

/// Descriptor for `ListWalletAccessResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listWalletAccessResponseDescriptor =
    $convert.base64Decode(
        'ChhMaXN0V2FsbGV0QWNjZXNzUmVzcG9uc2USTAoIYWNjZXNzZXMYASADKAsyMC5hcmJpdGVyLn'
        'VzZXJfYWdlbnQuc2RrX2NsaWVudC5XYWxsZXRBY2Nlc3NFbnRyeVIIYWNjZXNzZXM=');

@$core.Deprecated('Use requestDescriptor instead')
const Request$json = {
  '1': 'Request',
  '2': [
    {
      '1': 'connection_response',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.ConnectionResponse',
      '9': 0,
      '10': 'connectionResponse'
    },
    {
      '1': 'revoke',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.RevokeRequest',
      '9': 0,
      '10': 'revoke'
    },
    {
      '1': 'list',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'list'
    },
    {
      '1': 'grant_wallet_access',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.GrantWalletAccess',
      '9': 0,
      '10': 'grantWalletAccess'
    },
    {
      '1': 'revoke_wallet_access',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.RevokeWalletAccess',
      '9': 0,
      '10': 'revokeWalletAccess'
    },
    {
      '1': 'list_wallet_access',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'listWalletAccess'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode(
    'CgdSZXF1ZXN0EmQKE2Nvbm5lY3Rpb25fcmVzcG9uc2UYASABKAsyMS5hcmJpdGVyLnVzZXJfYW'
    'dlbnQuc2RrX2NsaWVudC5Db25uZWN0aW9uUmVzcG9uc2VIAFISY29ubmVjdGlvblJlc3BvbnNl'
    'EkYKBnJldm9rZRgCIAEoCzIsLmFyYml0ZXIudXNlcl9hZ2VudC5zZGtfY2xpZW50LlJldm9rZV'
    'JlcXVlc3RIAFIGcmV2b2tlEiwKBGxpc3QYAyABKAsyFi5nb29nbGUucHJvdG9idWYuRW1wdHlI'
    'AFIEbGlzdBJiChNncmFudF93YWxsZXRfYWNjZXNzGAQgASgLMjAuYXJiaXRlci51c2VyX2FnZW'
    '50LnNka19jbGllbnQuR3JhbnRXYWxsZXRBY2Nlc3NIAFIRZ3JhbnRXYWxsZXRBY2Nlc3MSZQoU'
    'cmV2b2tlX3dhbGxldF9hY2Nlc3MYBSABKAsyMS5hcmJpdGVyLnVzZXJfYWdlbnQuc2RrX2NsaW'
    'VudC5SZXZva2VXYWxsZXRBY2Nlc3NIAFIScmV2b2tlV2FsbGV0QWNjZXNzEkYKEmxpc3Rfd2Fs'
    'bGV0X2FjY2VzcxgGIAEoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eUgAUhBsaXN0V2FsbGV0QW'
    'NjZXNzQgkKB3BheWxvYWQ=');

@$core.Deprecated('Use responseDescriptor instead')
const Response$json = {
  '1': 'Response',
  '2': [
    {
      '1': 'connection_request',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.ConnectionRequest',
      '9': 0,
      '10': 'connectionRequest'
    },
    {
      '1': 'connection_cancel',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.ConnectionCancel',
      '9': 0,
      '10': 'connectionCancel'
    },
    {
      '1': 'revoke',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.RevokeResponse',
      '9': 0,
      '10': 'revoke'
    },
    {
      '1': 'list',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.ListResponse',
      '9': 0,
      '10': 'list'
    },
    {
      '1': 'list_wallet_access',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.arbiter.user_agent.sdk_client.ListWalletAccessResponse',
      '9': 0,
      '10': 'listWalletAccess'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode(
    'CghSZXNwb25zZRJhChJjb25uZWN0aW9uX3JlcXVlc3QYASABKAsyMC5hcmJpdGVyLnVzZXJfYW'
    'dlbnQuc2RrX2NsaWVudC5Db25uZWN0aW9uUmVxdWVzdEgAUhFjb25uZWN0aW9uUmVxdWVzdBJe'
    'ChFjb25uZWN0aW9uX2NhbmNlbBgCIAEoCzIvLmFyYml0ZXIudXNlcl9hZ2VudC5zZGtfY2xpZW'
    '50LkNvbm5lY3Rpb25DYW5jZWxIAFIQY29ubmVjdGlvbkNhbmNlbBJHCgZyZXZva2UYAyABKAsy'
    'LS5hcmJpdGVyLnVzZXJfYWdlbnQuc2RrX2NsaWVudC5SZXZva2VSZXNwb25zZUgAUgZyZXZva2'
    'USQQoEbGlzdBgEIAEoCzIrLmFyYml0ZXIudXNlcl9hZ2VudC5zZGtfY2xpZW50Lkxpc3RSZXNw'
    'b25zZUgAUgRsaXN0EmcKEmxpc3Rfd2FsbGV0X2FjY2VzcxgFIAEoCzI3LmFyYml0ZXIudXNlcl'
    '9hZ2VudC5zZGtfY2xpZW50Lkxpc3RXYWxsZXRBY2Nlc3NSZXNwb25zZUgAUhBsaXN0V2FsbGV0'
    'QWNjZXNzQgkKB3BheWxvYWQ=');
