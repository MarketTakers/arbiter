
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

@RoutePage()
class ClientDetails extends ConsumerWidget {
  final SdkClientEntry client;
  const ClientDetails({super.key, required this.client});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    throw UnimplementedError();
  }

}