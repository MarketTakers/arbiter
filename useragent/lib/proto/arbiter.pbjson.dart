// This is a generated file - do not edit.
//
// Generated from arbiter.proto.

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

import 'package:protobuf/well_known_types/google/protobuf/empty.pbjson.dart'
    as $3;
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pbjson.dart'
    as $4;

import 'client.pbjson.dart' as $0;
import 'evm.pbjson.dart' as $2;
import 'user_agent.pbjson.dart' as $1;

@$core.Deprecated('Use serverInfoDescriptor instead')
const ServerInfo$json = {
  '1': 'ServerInfo',
  '2': [
    {'1': 'version', '3': 1, '4': 1, '5': 9, '10': 'version'},
    {'1': 'cert_public_key', '3': 2, '4': 1, '5': 12, '10': 'certPublicKey'},
  ],
};

/// Descriptor for `ServerInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List serverInfoDescriptor = $convert.base64Decode(
    'CgpTZXJ2ZXJJbmZvEhgKB3ZlcnNpb24YASABKAlSB3ZlcnNpb24SJgoPY2VydF9wdWJsaWNfa2'
    'V5GAIgASgMUg1jZXJ0UHVibGljS2V5');

const $core.Map<$core.String, $core.dynamic> ArbiterServiceBase$json = {
  '1': 'ArbiterService',
  '2': [
    {
      '1': 'Client',
      '2': '.arbiter.client.ClientRequest',
      '3': '.arbiter.client.ClientResponse',
      '5': true,
      '6': true
    },
    {
      '1': 'UserAgent',
      '2': '.arbiter.user_agent.UserAgentRequest',
      '3': '.arbiter.user_agent.UserAgentResponse',
      '5': true,
      '6': true
    },
  ],
};

@$core.Deprecated('Use arbiterServiceDescriptor instead')
const $core.Map<$core.String, $core.Map<$core.String, $core.dynamic>>
    ArbiterServiceBase$messageJson = {
  '.arbiter.client.ClientRequest': $0.ClientRequest$json,
  '.arbiter.client.AuthChallengeRequest': $0.AuthChallengeRequest$json,
  '.arbiter.client.AuthChallengeSolution': $0.AuthChallengeSolution$json,
  '.arbiter.client.ClientResponse': $0.ClientResponse$json,
  '.arbiter.client.AuthChallenge': $0.AuthChallenge$json,
  '.arbiter.client.AuthOk': $0.AuthOk$json,
  '.arbiter.evm.EvmSignTransactionResponse': $2.EvmSignTransactionResponse$json,
  '.arbiter.evm.TransactionEvalError': $2.TransactionEvalError$json,
  '.google.protobuf.Empty': $3.Empty$json,
  '.arbiter.evm.NoMatchingGrantError': $2.NoMatchingGrantError$json,
  '.arbiter.evm.SpecificMeaning': $2.SpecificMeaning$json,
  '.arbiter.evm.EtherTransferMeaning': $2.EtherTransferMeaning$json,
  '.arbiter.evm.TokenTransferMeaning': $2.TokenTransferMeaning$json,
  '.arbiter.evm.TokenInfo': $2.TokenInfo$json,
  '.arbiter.evm.PolicyViolationsError': $2.PolicyViolationsError$json,
  '.arbiter.evm.EvalViolation': $2.EvalViolation$json,
  '.arbiter.evm.GasLimitExceededViolation': $2.GasLimitExceededViolation$json,
  '.arbiter.evm.EvmAnalyzeTransactionResponse':
      $2.EvmAnalyzeTransactionResponse$json,
  '.arbiter.client.ClientConnectError': $0.ClientConnectError$json,
  '.arbiter.user_agent.UserAgentRequest': $1.UserAgentRequest$json,
  '.arbiter.user_agent.AuthChallengeRequest': $1.AuthChallengeRequest$json,
  '.arbiter.user_agent.AuthChallengeSolution': $1.AuthChallengeSolution$json,
  '.arbiter.user_agent.UnsealStart': $1.UnsealStart$json,
  '.arbiter.user_agent.UnsealEncryptedKey': $1.UnsealEncryptedKey$json,
  '.arbiter.evm.EvmGrantCreateRequest': $2.EvmGrantCreateRequest$json,
  '.arbiter.evm.SharedSettings': $2.SharedSettings$json,
  '.google.protobuf.Timestamp': $4.Timestamp$json,
  '.arbiter.evm.TransactionRateLimit': $2.TransactionRateLimit$json,
  '.arbiter.evm.SpecificGrant': $2.SpecificGrant$json,
  '.arbiter.evm.EtherTransferSettings': $2.EtherTransferSettings$json,
  '.arbiter.evm.VolumeRateLimit': $2.VolumeRateLimit$json,
  '.arbiter.evm.TokenTransferSettings': $2.TokenTransferSettings$json,
  '.arbiter.evm.EvmGrantDeleteRequest': $2.EvmGrantDeleteRequest$json,
  '.arbiter.evm.EvmGrantListRequest': $2.EvmGrantListRequest$json,
  '.arbiter.user_agent.ClientConnectionResponse':
      $1.ClientConnectionResponse$json,
  '.arbiter.user_agent.UserAgentResponse': $1.UserAgentResponse$json,
  '.arbiter.user_agent.AuthChallenge': $1.AuthChallenge$json,
  '.arbiter.user_agent.AuthOk': $1.AuthOk$json,
  '.arbiter.user_agent.UnsealStartResponse': $1.UnsealStartResponse$json,
  '.arbiter.evm.WalletCreateResponse': $2.WalletCreateResponse$json,
  '.arbiter.evm.WalletEntry': $2.WalletEntry$json,
  '.arbiter.evm.WalletListResponse': $2.WalletListResponse$json,
  '.arbiter.evm.WalletList': $2.WalletList$json,
  '.arbiter.evm.EvmGrantCreateResponse': $2.EvmGrantCreateResponse$json,
  '.arbiter.evm.EvmGrantDeleteResponse': $2.EvmGrantDeleteResponse$json,
  '.arbiter.evm.EvmGrantListResponse': $2.EvmGrantListResponse$json,
  '.arbiter.evm.EvmGrantList': $2.EvmGrantList$json,
  '.arbiter.evm.GrantEntry': $2.GrantEntry$json,
  '.arbiter.user_agent.ClientConnectionRequest':
      $1.ClientConnectionRequest$json,
  '.arbiter.user_agent.ClientConnectionCancel': $1.ClientConnectionCancel$json,
};

/// Descriptor for `ArbiterService`. Decode as a `google.protobuf.ServiceDescriptorProto`.
final $typed_data.Uint8List arbiterServiceDescriptor = $convert.base64Decode(
    'Cg5BcmJpdGVyU2VydmljZRJLCgZDbGllbnQSHS5hcmJpdGVyLmNsaWVudC5DbGllbnRSZXF1ZX'
    'N0Gh4uYXJiaXRlci5jbGllbnQuQ2xpZW50UmVzcG9uc2UoATABElwKCVVzZXJBZ2VudBIkLmFy'
    'Yml0ZXIudXNlcl9hZ2VudC5Vc2VyQWdlbnRSZXF1ZXN0GiUuYXJiaXRlci51c2VyX2FnZW50Ll'
    'VzZXJBZ2VudFJlc3BvbnNlKAEwAQ==');
