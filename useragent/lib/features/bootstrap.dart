import 'dart:async';

import 'package:arbiter/providers/key.dart';
import 'package:flutter/src/widgets/framework.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:mtcore/markettakers.dart';

class Bootstrap extends HookConsumerWidget {
  final Completer<void> completer;

  const Bootstrap({required this.completer});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final container = ProviderScope.containerOf(context);
    final stages = useMemoized(() {
      return [KeyBootstrapper(ref: container)];
    }, []);
    final bootstrapper = useMemoized(
      () => Bootstrapper(stages: stages, onCompleted: completer),
      [stages],
    );
    return bootstrapper;
  }
}
