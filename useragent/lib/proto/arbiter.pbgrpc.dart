// This is a generated file - do not edit.
//
// Generated from arbiter.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'client.pb.dart' as $0;
import 'user_agent.pb.dart' as $1;

export 'arbiter.pb.dart';

@$pb.GrpcServiceName('arbiter.ArbiterService')
class ArbiterServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  ArbiterServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseStream<$0.ClientResponse> client(
    $async.Stream<$0.ClientRequest> request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(_$client, request, options: options);
  }

  $grpc.ResponseStream<$1.UserAgentResponse> userAgent(
    $async.Stream<$1.UserAgentRequest> request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(_$userAgent, request, options: options);
  }

  // method descriptors

  static final _$client =
      $grpc.ClientMethod<$0.ClientRequest, $0.ClientResponse>(
          '/arbiter.ArbiterService/Client',
          ($0.ClientRequest value) => value.writeToBuffer(),
          $0.ClientResponse.fromBuffer);
  static final _$userAgent =
      $grpc.ClientMethod<$1.UserAgentRequest, $1.UserAgentResponse>(
          '/arbiter.ArbiterService/UserAgent',
          ($1.UserAgentRequest value) => value.writeToBuffer(),
          $1.UserAgentResponse.fromBuffer);
}

@$pb.GrpcServiceName('arbiter.ArbiterService')
abstract class ArbiterServiceBase extends $grpc.Service {
  $core.String get $name => 'arbiter.ArbiterService';

  ArbiterServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.ClientRequest, $0.ClientResponse>(
        'Client',
        client,
        true,
        true,
        ($core.List<$core.int> value) => $0.ClientRequest.fromBuffer(value),
        ($0.ClientResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$1.UserAgentRequest, $1.UserAgentResponse>(
        'UserAgent',
        userAgent,
        true,
        true,
        ($core.List<$core.int> value) => $1.UserAgentRequest.fromBuffer(value),
        ($1.UserAgentResponse value) => value.writeToBuffer()));
  }

  $async.Stream<$0.ClientResponse> client(
      $grpc.ServiceCall call, $async.Stream<$0.ClientRequest> request);

  $async.Stream<$1.UserAgentResponse> userAgent(
      $grpc.ServiceCall call, $async.Stream<$1.UserAgentRequest> request);
}
