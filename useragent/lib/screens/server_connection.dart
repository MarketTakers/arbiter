import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:arbiter/router.gr.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class ServerConnectionScreen extends HookConsumerWidget {
  const ServerConnectionScreen({super.key, this.arbiterUrl});

  final String? arbiterUrl;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final connectionState = ref.watch(connectionManagerProvider);

    if (connectionState.value != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        context.router.replace(const VaultSetupRoute());
      });
    }

    final body = switch (connectionState) {
      AsyncLoading() => const CircularProgressIndicator(),
      AsyncError(:final error) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            'Connection failed',
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
          SizedBox(height: 1.5.h),
          Text('$error', textAlign: TextAlign.center),
          SizedBox(height: 2.h),

          TextButton(
            onPressed: () {
              context.router.replace(const ServerInfoSetupRoute());
            },
            child: const Text('Back to server setup'),
          ),
        ],
      ),
      _ => const CircularProgressIndicator(),
    };

    return Scaffold(
      appBar: AppBar(title: const Text('Connecting')),
      body: Center(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8.w),
          child: body,
        ),
      ),
    );
  }
}
