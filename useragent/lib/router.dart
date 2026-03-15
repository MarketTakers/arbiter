import 'package:arbiter/screens/dashboard/about.dart';
import 'package:arbiter/screens/dashboard/calc.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_adaptive_scaffold/flutter_adaptive_scaffold.dart';

import 'router.gr.dart';

@AutoRouterConfig(generateForDir: ['lib/screens'])
class Router extends RootStackRouter {
  @override
  List<AutoRoute> get routes => [
    AutoRoute(page: Bootstrap.page, path: '/bootstrap', initial: true),

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
