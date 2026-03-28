// dart format width=80
// GENERATED CODE - DO NOT MODIFY BY HAND

// **************************************************************************
// AutoRouterGenerator
// **************************************************************************

// ignore_for_file: type=lint
// coverage:ignore-file

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:arbiter/proto/user_agent.pb.dart' as _i15;
import 'package:arbiter/screens/bootstrap.dart' as _i2;
import 'package:arbiter/screens/dashboard.dart' as _i7;
import 'package:arbiter/screens/dashboard/about.dart' as _i1;
import 'package:arbiter/screens/dashboard/clients/details.dart' as _i3;
import 'package:arbiter/screens/dashboard/clients/details/client_details.dart'
    as _i4;
import 'package:arbiter/screens/dashboard/clients/table.dart' as _i5;
import 'package:arbiter/screens/dashboard/evm/evm.dart' as _i9;
import 'package:arbiter/screens/dashboard/evm/grants/grant_create.dart' as _i6;
import 'package:arbiter/screens/dashboard/evm/grants/grants.dart' as _i8;
import 'package:arbiter/screens/server_connection.dart' as _i10;
import 'package:arbiter/screens/server_info_setup.dart' as _i11;
import 'package:arbiter/screens/vault_setup.dart' as _i12;
import 'package:auto_route/auto_route.dart' as _i13;
import 'package:flutter/material.dart' as _i14;

/// generated route for
/// [_i1.AboutScreen]
class AboutRoute extends _i13.PageRouteInfo<void> {
  const AboutRoute({List<_i13.PageRouteInfo>? children})
    : super(AboutRoute.name, initialChildren: children);

  static const String name = 'AboutRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i1.AboutScreen();
    },
  );
}

/// generated route for
/// [_i2.Bootstrap]
class Bootstrap extends _i13.PageRouteInfo<void> {
  const Bootstrap({List<_i13.PageRouteInfo>? children})
    : super(Bootstrap.name, initialChildren: children);

  static const String name = 'Bootstrap';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i2.Bootstrap();
    },
  );
}

/// generated route for
/// [_i3.ClientDetails]
class ClientDetails extends _i13.PageRouteInfo<ClientDetailsArgs> {
  ClientDetails({
    _i14.Key? key,
    required _i15.SdkClientEntry client,
    List<_i13.PageRouteInfo>? children,
  }) : super(
         ClientDetails.name,
         args: ClientDetailsArgs(key: key, client: client),
         initialChildren: children,
       );

  static const String name = 'ClientDetails';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      final args = data.argsAs<ClientDetailsArgs>();
      return _i3.ClientDetails(key: args.key, client: args.client);
    },
  );
}

class ClientDetailsArgs {
  const ClientDetailsArgs({this.key, required this.client});

  final _i14.Key? key;

  final _i15.SdkClientEntry client;

  @override
  String toString() {
    return 'ClientDetailsArgs{key: $key, client: $client}';
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other is! ClientDetailsArgs) return false;
    return key == other.key && client == other.client;
  }

  @override
  int get hashCode => key.hashCode ^ client.hashCode;
}

/// generated route for
/// [_i4.ClientDetailsScreen]
class ClientDetailsRoute extends _i13.PageRouteInfo<ClientDetailsRouteArgs> {
  ClientDetailsRoute({
    _i14.Key? key,
    required int clientId,
    List<_i13.PageRouteInfo>? children,
  }) : super(
         ClientDetailsRoute.name,
         args: ClientDetailsRouteArgs(key: key, clientId: clientId),
         rawPathParams: {'clientId': clientId},
         initialChildren: children,
       );

  static const String name = 'ClientDetailsRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      final pathParams = data.inheritedPathParams;
      final args = data.argsAs<ClientDetailsRouteArgs>(
        orElse: () =>
            ClientDetailsRouteArgs(clientId: pathParams.getInt('clientId')),
      );
      return _i4.ClientDetailsScreen(key: args.key, clientId: args.clientId);
    },
  );
}

class ClientDetailsRouteArgs {
  const ClientDetailsRouteArgs({this.key, required this.clientId});

  final _i14.Key? key;

  final int clientId;

  @override
  String toString() {
    return 'ClientDetailsRouteArgs{key: $key, clientId: $clientId}';
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other is! ClientDetailsRouteArgs) return false;
    return key == other.key && clientId == other.clientId;
  }

  @override
  int get hashCode => key.hashCode ^ clientId.hashCode;
}

/// generated route for
/// [_i5.ClientsScreen]
class ClientsRoute extends _i13.PageRouteInfo<void> {
  const ClientsRoute({List<_i13.PageRouteInfo>? children})
    : super(ClientsRoute.name, initialChildren: children);

  static const String name = 'ClientsRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i5.ClientsScreen();
    },
  );
}

/// generated route for
/// [_i6.CreateEvmGrantScreen]
class CreateEvmGrantRoute extends _i13.PageRouteInfo<void> {
  const CreateEvmGrantRoute({List<_i13.PageRouteInfo>? children})
    : super(CreateEvmGrantRoute.name, initialChildren: children);

  static const String name = 'CreateEvmGrantRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i6.CreateEvmGrantScreen();
    },
  );
}

/// generated route for
/// [_i7.DashboardRouter]
class DashboardRouter extends _i13.PageRouteInfo<void> {
  const DashboardRouter({List<_i13.PageRouteInfo>? children})
    : super(DashboardRouter.name, initialChildren: children);

  static const String name = 'DashboardRouter';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i7.DashboardRouter();
    },
  );
}

/// generated route for
/// [_i8.EvmGrantsScreen]
class EvmGrantsRoute extends _i13.PageRouteInfo<void> {
  const EvmGrantsRoute({List<_i13.PageRouteInfo>? children})
    : super(EvmGrantsRoute.name, initialChildren: children);

  static const String name = 'EvmGrantsRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i8.EvmGrantsScreen();
    },
  );
}

/// generated route for
/// [_i9.EvmScreen]
class EvmRoute extends _i13.PageRouteInfo<void> {
  const EvmRoute({List<_i13.PageRouteInfo>? children})
    : super(EvmRoute.name, initialChildren: children);

  static const String name = 'EvmRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i9.EvmScreen();
    },
  );
}

/// generated route for
/// [_i10.ServerConnectionScreen]
class ServerConnectionRoute
    extends _i13.PageRouteInfo<ServerConnectionRouteArgs> {
  ServerConnectionRoute({
    _i14.Key? key,
    String? arbiterUrl,
    List<_i13.PageRouteInfo>? children,
  }) : super(
         ServerConnectionRoute.name,
         args: ServerConnectionRouteArgs(key: key, arbiterUrl: arbiterUrl),
         initialChildren: children,
       );

  static const String name = 'ServerConnectionRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      final args = data.argsAs<ServerConnectionRouteArgs>(
        orElse: () => const ServerConnectionRouteArgs(),
      );
      return _i10.ServerConnectionScreen(
        key: args.key,
        arbiterUrl: args.arbiterUrl,
      );
    },
  );
}

class ServerConnectionRouteArgs {
  const ServerConnectionRouteArgs({this.key, this.arbiterUrl});

  final _i14.Key? key;

  final String? arbiterUrl;

  @override
  String toString() {
    return 'ServerConnectionRouteArgs{key: $key, arbiterUrl: $arbiterUrl}';
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other is! ServerConnectionRouteArgs) return false;
    return key == other.key && arbiterUrl == other.arbiterUrl;
  }

  @override
  int get hashCode => key.hashCode ^ arbiterUrl.hashCode;
}

/// generated route for
/// [_i11.ServerInfoSetupScreen]
class ServerInfoSetupRoute extends _i13.PageRouteInfo<void> {
  const ServerInfoSetupRoute({List<_i13.PageRouteInfo>? children})
    : super(ServerInfoSetupRoute.name, initialChildren: children);

  static const String name = 'ServerInfoSetupRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i11.ServerInfoSetupScreen();
    },
  );
}

/// generated route for
/// [_i12.VaultSetupScreen]
class VaultSetupRoute extends _i13.PageRouteInfo<void> {
  const VaultSetupRoute({List<_i13.PageRouteInfo>? children})
    : super(VaultSetupRoute.name, initialChildren: children);

  static const String name = 'VaultSetupRoute';

  static _i13.PageInfo page = _i13.PageInfo(
    name,
    builder: (data) {
      return const _i12.VaultSetupScreen();
    },
  );
}
