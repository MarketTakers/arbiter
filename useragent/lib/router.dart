import 'dart:async';

import 'package:arbiter/features/bootstrap.dart';
import 'package:arbiter/home.dart';
import 'package:flutter/src/widgets/async.dart';
import 'package:flutter/src/widgets/framework.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

final _bootstapCompleter = Completer<void>();

class Router extends HookConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final bootstrapper = useMemoized(
      () => Bootstrap(completer: _bootstapCompleter),
    );
    final bootstrapFuture = useFuture(_bootstapCompleter.future);

    switch (bootstrapFuture.connectionState) {
      case ConnectionState.none ||
          ConnectionState.waiting ||
          ConnectionState.active:
        return bootstrapper;

      case ConnectionState.done:
        break;
    }

    return Home();
  }
}
