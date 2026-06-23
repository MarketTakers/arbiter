import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

@RoutePage()
class ClientDetails extends ConsumerWidget {
  final ua_sdk.Entry client;
  const ClientDetails({super.key, required this.client});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    throw UnimplementedError();
  }
}
