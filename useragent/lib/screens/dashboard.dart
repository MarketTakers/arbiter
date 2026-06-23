import 'package:arbiter/features/callouts/callout_manager.dart';
import 'package:arbiter/features/callouts/show_callout_list.dart';
import 'package:arbiter/router.gr.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_adaptive_scaffold/flutter_adaptive_scaffold.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

const breakpoints = MaterialAdaptiveBreakpoints();

final routes = [
  const EvmRoute(),
  const ClientsRoute(),
  const EvmGrantsRoute(),
  const AboutRoute(),
];

@RoutePage()
class DashboardRouter extends StatelessWidget {
  const DashboardRouter({super.key});

  @override
  Widget build(BuildContext context) {
    final title = Container(
      margin: const EdgeInsets.all(16),
      child: const Text(
        "Arbiter",
        style: TextStyle(fontWeight: FontWeight.w800),
      ),
    );

    return AutoTabsRouter(
      routes: routes,
      transitionBuilder: (context, child, animation) =>
          FadeTransition(opacity: animation, child: child),
      builder: (context, child) {
        final tabsRouter = AutoTabsRouter.of(context);
        final currentActive = tabsRouter.activeIndex;
        return AdaptiveScaffold(
          destinations: const [
            NavigationDestination(
              icon: Icon(Icons.account_balance_wallet_outlined),
              selectedIcon: Icon(Icons.account_balance_wallet),
              label: "Wallets",
            ),
            NavigationDestination(
              icon: Icon(Icons.devices_other_outlined),
              selectedIcon: Icon(Icons.devices_other),
              label: "Clients",
            ),
            NavigationDestination(
              icon: Icon(Icons.policy_outlined),
              selectedIcon: Icon(Icons.policy),
              label: "Grants",
            ),
            NavigationDestination(
              icon: Icon(Icons.info_outline),
              selectedIcon: Icon(Icons.info),
              label: "About",
            ),
          ],
          body: (ctx) => child,
          onSelectedIndexChange: (index) {
            tabsRouter.navigate(routes[index]);
          },
          leadingExtendedNavRail: title,
          leadingUnextendedNavRail: title,
          selectedIndex: currentActive,
          transitionDuration: const Duration(milliseconds: 800),
          internalAnimations: true,

          trailingNavRail: const _CalloutBell(),
        );
      },
    );
  }
}

class _CalloutBell extends ConsumerWidget {
  const _CalloutBell();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final count = ref.watch(calloutManagerProvider.select((map) => map.length));

    return IconButton(
      onPressed: () => showCalloutList(context, ref),
      icon: Stack(
        clipBehavior: Clip.none,
        children: [
          Icon(
            count > 0 ? Icons.notifications : Icons.notifications_outlined,
            color: Palette.ink,
          ),
          if (count > 0)
            Positioned(
              top: -2,
              right: -4,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                decoration: BoxDecoration(
                  color: Colors.green,
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(
                  count > 99 ? '99+' : '$count',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 9,
                    fontWeight: FontWeight.w800,
                    height: 1.2,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
