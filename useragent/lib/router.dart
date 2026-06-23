import 'package:auto_route/auto_route.dart';

import 'router.gr.dart';

@AutoRouterConfig(generateForDir: ['lib/screens'])
class Router extends RootStackRouter {
  @override
  List<AutoRoute> get routes => [
    AutoRoute(page: Bootstrap.page, path: '/bootstrap', initial: true),
    AutoRoute(page: ServerInfoSetupRoute.page, path: '/server-info'),
    AutoRoute(page: ServerConnectionRoute.page, path: '/server-connection'),
    AutoRoute(page: VaultSetupRoute.page, path: '/vault'),
    AutoRoute(page: ClientDetailsRoute.page, path: '/clients/:clientId'),
    AutoRoute(page: CreateEvmGrantRoute.page, path: '/evm-grants/create'),

    AutoRoute(
      page: DashboardRouter.page,
      path: '/dashboard',
      children: [
        AutoRoute(page: EvmRoute.page, path: 'evm'),
        AutoRoute(page: ClientsRoute.page, path: 'clients'),
        AutoRoute(page: EvmGrantsRoute.page, path: 'grants'),
        AutoRoute(page: AboutRoute.page, path: 'about'),
      ],
    ),
  ];
}
