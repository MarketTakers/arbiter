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

import 'package:protobuf/protobuf.dart' as $pb;

import 'arbiter.pbjson.dart';
import 'client.pb.dart' as $0;
import 'user_agent.pb.dart' as $1;

export 'arbiter.pb.dart';

abstract class ArbiterServiceBase extends $pb.GeneratedService {
  $async.Future<$0.ClientResponse> client(
      $pb.ServerContext ctx, $0.ClientRequest request);
  $async.Future<$1.UserAgentResponse> userAgent(
      $pb.ServerContext ctx, $1.UserAgentRequest request);

  $pb.GeneratedMessage createRequest($core.String methodName) {
    switch (methodName) {
      case 'Client':
        return $0.ClientRequest();
      case 'UserAgent':
        return $1.UserAgentRequest();
      default:
        throw $core.ArgumentError('Unknown method: $methodName');
    }
  }

  $async.Future<$pb.GeneratedMessage> handleCall($pb.ServerContext ctx,
      $core.String methodName, $pb.GeneratedMessage request) {
    switch (methodName) {
      case 'Client':
        return client(ctx, request as $0.ClientRequest);
      case 'UserAgent':
        return userAgent(ctx, request as $1.UserAgentRequest);
      default:
        throw $core.ArgumentError('Unknown method: $methodName');
    }
  }

  $core.Map<$core.String, $core.dynamic> get $json => ArbiterServiceBase$json;
  $core.Map<$core.String, $core.Map<$core.String, $core.dynamic>>
      get $messageJson => ArbiterServiceBase$messageJson;
}
