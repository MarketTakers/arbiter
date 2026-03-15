import 'package:auto_route/auto_route.dart';

import 'router.gr.dart';

@AutoRouterConfig(generateForDir: ['lib/screens'])
class Router extends RootStackRouter {
  @override
  List<AutoRoute> get routes => [
    AutoRoute(page: Bootstrap.page, path: '/bootstrap', initial: true),
    AutoRoute(page: ServerInfoSetupRoute.page, path: '/server-info'),
    AutoRoute(page: ServerConnectionRoute.page, path: '/server-connection'),

    AutoRoute(
      page: DashboardRouter.page,
      path: '/dashboard',
      children: [
        AutoRoute(page: AboutRoute.page, path: 'about'),
        AutoRoute(page: CalcRoute.page, path: 'calc'),
      ],
    ),
  ];
}
