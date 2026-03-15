// dart format width=80
// GENERATED CODE - DO NOT MODIFY BY HAND

// **************************************************************************
// AutoRouterGenerator
// **************************************************************************

// ignore_for_file: type=lint
// coverage:ignore-file

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:arbiter/screens/bootstrap.dart' as _i2;
import 'package:arbiter/screens/dashboard.dart' as _i4;
import 'package:arbiter/screens/dashboard/about.dart' as _i1;
import 'package:arbiter/screens/dashboard/calc.dart' as _i3;
import 'package:auto_route/auto_route.dart' as _i5;

/// generated route for
/// [_i1.AboutScreen]
class AboutRoute extends _i5.PageRouteInfo<void> {
  const AboutRoute({List<_i5.PageRouteInfo>? children})
    : super(AboutRoute.name, initialChildren: children);

  static const String name = 'AboutRoute';

  static _i5.PageInfo page = _i5.PageInfo(
    name,
    builder: (data) {
      return _i1.AboutScreen();
    },
  );
}

/// generated route for
/// [_i2.Bootstrap]
class Bootstrap extends _i5.PageRouteInfo<void> {
  const Bootstrap({List<_i5.PageRouteInfo>? children})
    : super(Bootstrap.name, initialChildren: children);

  static const String name = 'Bootstrap';

  static _i5.PageInfo page = _i5.PageInfo(
    name,
    builder: (data) {
      return const _i2.Bootstrap();
    },
  );
}

/// generated route for
/// [_i3.CalcScreen]
class CalcRoute extends _i5.PageRouteInfo<void> {
  const CalcRoute({List<_i5.PageRouteInfo>? children})
    : super(CalcRoute.name, initialChildren: children);

  static const String name = 'CalcRoute';

  static _i5.PageInfo page = _i5.PageInfo(
    name,
    builder: (data) {
      return const _i3.CalcScreen();
    },
  );
}

/// generated route for
/// [_i4.DashboardRouter]
class DashboardRouter extends _i5.PageRouteInfo<void> {
  const DashboardRouter({List<_i5.PageRouteInfo>? children})
    : super(DashboardRouter.name, initialChildren: children);

  static const String name = 'DashboardRouter';

  static _i5.PageInfo page = _i5.PageInfo(
    name,
    builder: (data) {
      return const _i4.DashboardRouter();
    },
  );
}
