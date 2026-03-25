import 'package:arbiter/providers/sdk_clients/details.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/client_details_content.dart';
import 'package:arbiter/screens/dashboard/clients/details/widgets/client_details_state_panel.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

@RoutePage()
class ClientDetailsScreen extends ConsumerWidget {
  const ClientDetailsScreen({super.key, @pathParam required this.clientId});

  final int clientId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final clientAsync = ref.watch(clientDetailsProvider(clientId));
    return Scaffold(
      body: SafeArea(
        child: clientAsync.when(
          data: (client) =>
              _ClientDetailsState(clientId: clientId, client: client),
          error: (error, _) => ClientDetailsStatePanel(
            title: 'Client unavailable',
            body: error.toString(),
            icon: Icons.sync_problem,
          ),
          loading: () => const ClientDetailsStatePanel(
            title: 'Loading client',
            body: 'Pulling client details from Arbiter.',
            icon: Icons.hourglass_top,
          ),
        ),
      ),
    );
  }
}

class _ClientDetailsState extends StatelessWidget {
  const _ClientDetailsState({required this.clientId, required this.client});

  final int clientId;
  final SdkClientEntry? client;

  @override
  Widget build(BuildContext context) {
    if (client == null) {
      return const ClientDetailsStatePanel(
        title: 'Client not found',
        body: 'The selected SDK client is no longer available.',
        icon: Icons.person_off_outlined,
      );
    }
    return ClientDetailsContent(clientId: clientId, client: client!);
  }
}
