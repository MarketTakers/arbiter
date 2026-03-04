import 'package:flutter_adaptive_scaffold/flutter_adaptive_scaffold.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:sizer/sizer.dart';

const transitionDuration = Duration(milliseconds: 800);

class AdaptiveBuilders {
  WidgetBuilder? buildSmall;
  WidgetBuilder? buildMediumLarge;
  WidgetBuilder? buildLarge;
  WidgetBuilder? buildExtraLarge;
  WidgetBuilder? build;

  AdaptiveBuilders({
    this.buildSmall,
    this.buildMediumLarge,
    this.buildLarge,
    this.buildExtraLarge,
    this.build,
  });
}

class Destination {
  final String label;
  final String? tooltip;
  final Icon icon;
  final Icon? selectedIcon;

  final AdaptiveBuilders main;
  final AdaptiveBuilders? secondary;

  Destination({
    required this.label,
    required this.icon,
    this.selectedIcon,
    required this.main,
    this.secondary,
    this.tooltip,
  });
}

class Switcher extends StatelessWidget {
  final Widget? child;

  const Switcher({super.key, this.child});
  @override
  Widget build(BuildContext context) {
    return AnimatedSwitcher(
      duration: Duration(
        milliseconds: transitionDuration.inMilliseconds ~/ 100,
      ),
      transitionBuilder: (child, animation) {
        return FadeTransition(opacity: animation, child: child);
      },
      child: child,
    );
  }
}

WidgetBuilder? patchAnimated(WidgetBuilder? input) {
  if (input == null) return null;
  return (context) => Switcher(child: input(context));
}

class HomeRouter extends HookWidget {
  final List<Destination> destinations;

  HomeRouter({super.key, required this.destinations})
    : assert(destinations.isNotEmpty);

  @override
  Widget build(BuildContext context) {
    final selectedIndex = useState(0);
    final destination = useMemoized(() => destinations[selectedIndex.value], [
      selectedIndex.value,
    ]);
    final dispatcher = useMemoized(() => destination.main, [
      selectedIndex.value,
    ]);
    final secondaryDispatcher = useMemoized(() => destination.secondary, [
      selectedIndex.value,
    ]);

    return AdaptiveScaffold(
      destinations: destinations
          .map(
            (destination) => NavigationDestination(
              label: destination.label,
              icon: destination.icon,
              selectedIcon: destination.selectedIcon,
              tooltip: destination.tooltip,
            ),
          )
          .toList(),

      selectedIndex: selectedIndex.value,
      onSelectedIndexChange: (index) => selectedIndex.value = index,
      useDrawer: true,

      smallBody: patchAnimated(dispatcher.buildSmall),
      body: patchAnimated(dispatcher.build),
      mediumLargeBody: patchAnimated(dispatcher.buildMediumLarge),
      largeBody: patchAnimated(dispatcher.buildLarge),
      extraLargeBody: patchAnimated(dispatcher.buildExtraLarge),

      smallSecondaryBody: patchAnimated(secondaryDispatcher?.buildSmall),
      secondaryBody: patchAnimated(secondaryDispatcher?.build),
      mediumLargeSecondaryBody: patchAnimated(
        secondaryDispatcher?.buildMediumLarge,
      ),
      largeSecondaryBody: patchAnimated(secondaryDispatcher?.buildLarge),
      extraLargeSecondaryBody: patchAnimated(
        secondaryDispatcher?.buildExtraLarge,
      ),
    );
  }
}
