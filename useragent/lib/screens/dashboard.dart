import 'package:arbiter/router.gr.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_adaptive_scaffold/flutter_adaptive_scaffold.dart';

const breakpoints = MaterialAdaptiveBreakpoints();

final routes = [EvmRoute(), AboutRoute()];

@RoutePage()
class DashboardRouter extends StatelessWidget {
  const DashboardRouter({super.key});

  @override
  Widget build(BuildContext context) {
    return AutoTabsRouter(
      routes: routes,
      transitionBuilder: (context, child, animation) => FadeTransition(
        opacity: animation,
        // the passed child is technically our animated selected-tab page
        child: child,
      ),
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
              icon: Icon(Icons.info_outline),
              selectedIcon: Icon(Icons.info),
              label: "About",
            ),
          ],
          body: (ctx) => child,
          onSelectedIndexChange: (index) {
            tabsRouter.navigate(routes[index]);
          },
          selectedIndex: currentActive,
          transitionDuration: const Duration(milliseconds: 800),
          internalAnimations: true,
        );
      },
    );
  }
}
