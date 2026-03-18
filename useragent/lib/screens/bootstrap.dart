import 'dart:async';

import 'package:arbiter/providers/key.dart';
import 'package:arbiter/router.gr.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:mtcore/markettakers.dart';

@RoutePage()
class Bootstrap extends HookConsumerWidget {
  const Bootstrap({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final container = ProviderScope.containerOf(context);
    final completer = useMemoized(() {
      final completer = Completer<void>();
      completer.future.then((_) async {
        if (context.mounted) {
          final router = AutoRouter.of(context);
          router.replace(const ServerInfoSetupRoute());
        }
      });

      return completer;
    }, []);
    final stages = useMemoized(() {
      return [KeyBootstrapper(ref: container)];
    }, []);
    final bootstrapper = useMemoized(
      () => Bootstrapper(stages: stages, onCompleted: completer),
      [stages],
    );

    return Scaffold(body: bootstrapper);
  }
}
