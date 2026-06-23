import 'package:arbiter/features/connection/arbiter_url.dart';
import 'package:arbiter/features/connection/server_info_storage.dart';
import 'package:arbiter/providers/connection/bootstrap_token.dart';
import 'package:arbiter/providers/server_info.dart';
import 'package:arbiter/router.gr.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class ServerInfoSetupScreen extends HookConsumerWidget {
  const ServerInfoSetupScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final storedServerInfo = ref.watch(serverInfoProvider);
    final resolvedServerInfo = storedServerInfo.asData?.value;
    final controller = useTextEditingController();
    final errorText = useState<String?>(null);
    final isSaving = useState(false);

    useEffect(() {
      final serverInfo = resolvedServerInfo;
      if (serverInfo != null) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (context.mounted) {
            context.router.replace(ServerConnectionRoute());
          }
        });
      }
      return null;
    }, [context, resolvedServerInfo]);

    Future<void> saveRemoteServerInfo() async {
      errorText.value = null;
      isSaving.value = true;

      try {
        final arbiterUrl = ArbiterUrl.parse(controller.text.trim());

        // set token before triggering reconnection by updating server info
        if (arbiterUrl.bootstrapToken != null) {
          ref
              .read(bootstrapTokenProvider.notifier)
              .set(arbiterUrl.bootstrapToken!);
        }

        await ref
            .read(serverInfoProvider.notifier)
            .save(
              address: arbiterUrl.host,
              port: arbiterUrl.port,
              caCert: arbiterUrl.caCert,
            );

        if (context.mounted) {
          context.router.replace(
            ServerConnectionRoute(arbiterUrl: controller.text.trim()),
          );
        }
      } on FormatException catch (error) {
        errorText.value = error.message;
      } catch (_) {
        errorText.value = 'Failed to store connection settings.';
      } finally {
        isSaving.value = false;
      }
    }

    if (storedServerInfo.isLoading) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }

    if (storedServerInfo.hasError) {
      return Scaffold(
        appBar: AppBar(title: const Text('Server Info Setup')),
        body: Center(
          child: Padding(
            padding: EdgeInsets.symmetric(horizontal: 6.w),
            child: Text(
              'Failed to load stored server info.',
              style: Theme.of(context).textTheme.bodyLarge,
              textAlign: TextAlign.center,
            ),
          ),
        ),
      );
    }

    final serverInfo = resolvedServerInfo;
    if (serverInfo != null) {
      return _RedirectingView(serverInfo: serverInfo);
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Server Info Setup')),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final useRowLayout = constraints.maxWidth > constraints.maxHeight;
          final gap = 2.h;
          final horizontalPadding = 6.w;
          final verticalPadding = 3.h;
          final options = [
            const _OptionCard(
              title: 'Local',
              subtitle:
                  'Will start and connect to a local service in a future update.',
              enabled: false,
              child: SizedBox.shrink(),
            ),
            _OptionCard(
              title: 'Remote',
              subtitle:
                  'Paste an Arbiter URL to store the server address, port, and CA fingerprint.',
              child: _RemoteConnectionForm(
                controller: controller,
                errorText: errorText.value,
                isSaving: isSaving.value,
                onSave: saveRemoteServerInfo,
              ),
            ),
          ];

          return ListView(
            padding: EdgeInsets.symmetric(
              horizontal: horizontalPadding,
              vertical: verticalPadding,
            ),
            children: [
              _SetupHeader(gap: gap),
              SizedBox(height: gap),
              useRowLayout
                  ? Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Expanded(child: options[0]),
                        SizedBox(width: 3.w),
                        Expanded(child: options[1]),
                      ],
                    )
                  : Column(
                      children: [
                        options[0],
                        SizedBox(height: gap),
                        options[1],
                      ],
                    ),
            ],
          );
        },
      ),
    );
  }
}

class _SetupHeader extends StatelessWidget {
  const _SetupHeader({required this.gap});

  final double gap;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Choose how this user agent should reach Arbiter.',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        SizedBox(height: gap * 0.5),
        Text(
          'Remote accepts the shareable Arbiter URL emitted by the server.',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
      ],
    );
  }
}

class _RedirectingView extends StatelessWidget {
  const _RedirectingView({required this.serverInfo});

  final StoredServerInfo serverInfo;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator(),
            SizedBox(height: 2.h),
            Text('Using saved server ${serverInfo.address}:${serverInfo.port}'),
          ],
        ),
      ),
    );
  }
}

class _RemoteConnectionForm extends StatelessWidget {
  const _RemoteConnectionForm({
    required this.controller,
    required this.errorText,
    required this.isSaving,
    required this.onSave,
  });

  final TextEditingController controller;
  final String? errorText;
  final bool isSaving;
  final VoidCallback onSave;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TextField(
          controller: controller,
          decoration: const InputDecoration(
            border: OutlineInputBorder(),
            labelText: 'Arbiter URL',
            hintText: 'arbiter://host:port?cert=...',
          ),
          minLines: 2,
          maxLines: 4,
        ),
        if (errorText != null) ...[
          SizedBox(height: 1.5.h),
          Text(
            errorText!,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
        ],
        SizedBox(height: 2.h),
        Align(
          alignment: Alignment.centerLeft,
          child: FilledButton(
            onPressed: isSaving ? null : onSave,
            child: Text(isSaving ? 'Saving...' : 'Save connection'),
          ),
        ),
      ],
    );
  }
}

class _OptionCard extends StatelessWidget {
  const _OptionCard({
    required this.title,
    required this.subtitle,
    required this.child,
    this.enabled = true,
  });

  final String title;
  final String subtitle;
  final Widget child;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      child: Padding(
        padding: EdgeInsets.all(2.h),
        child: Opacity(
          opacity: enabled ? 1 : 0.55,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(title, style: theme.textTheme.titleLarge),
                  if (!enabled) ...[
                    SizedBox(width: 2.w),
                    const Chip(label: Text('Coming soon')),
                  ],
                ],
              ),
              SizedBox(height: 1.h),
              Text(subtitle, style: theme.textTheme.bodyMedium),
              if (enabled) ...[SizedBox(height: 2.h), child],
            ],
          ),
        ),
      ),
    );
  }
}
