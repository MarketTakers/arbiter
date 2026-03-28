import 'dart:math' as math;

import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:arbiter/router.gr.dart';
import 'package:arbiter/providers/sdk_clients/list.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:arbiter/widgets/cream_frame.dart';
import 'package:arbiter/widgets/state_panel.dart';
import 'package:sizer/sizer.dart';

// ─── Column width getters ─────────────────────────────────────────────────────

double get _accentStripWidth => 0.8.w;
double get _cellHPad => 1.8.w;
double get _colGap => 1.8.w;
double get _idColWidth => 8.w;
double get _nameColWidth => 20.w;
double get _versionColWidth => 12.w;
double get _registeredColWidth => 18.w;
double get _chevronColWidth => 4.w;
double get _tableMinWidth => 72.w;

// ─── Helpers ──────────────────────────────────────────────────────────────────

Color _accentColor(List<int> bytes) {
  final seed = bytes.fold<int>(0, (v, b) => v + b);
  final hue = (seed * 17) % 360;
  return HSLColor.fromAHSL(1, hue.toDouble(), 0.68, 0.54).toColor();
}

// ed25519 public keys are always 32 bytes (64 hex chars); guard is defensive.
String _shortPubkey(List<int> bytes) {
  final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  if (hex.length < 12) return '0x$hex';
  return '0x${hex.substring(0, 8)}...${hex.substring(hex.length - 4)}';
}

String _fullPubkey(List<int> bytes) {
  final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  return '0x$hex';
}

String _formatDate(int unixSecs) {
  final dt = DateTime.fromMillisecondsSinceEpoch(unixSecs * 1000).toLocal();
  return '${dt.year}-'
      '${dt.month.toString().padLeft(2, '0')}-'
      '${dt.day.toString().padLeft(2, '0')}';
}

String _formatError(Object error) {
  final message = error.toString();
  if (message.startsWith('Exception: ')) {
    return message.substring('Exception: '.length);
  }
  return message;
}

// ─── Header ───────────────────────────────────────────────────────────────────

class _Header extends StatelessWidget {
  const _Header({required this.isBusy, required this.onRefresh});

  final bool isBusy;
  final Future<void> Function() onRefresh;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding: EdgeInsets.symmetric(horizontal: 1.6.w, vertical: 1.2.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(18),
        color: Palette.cream,
        border: Border.all(color: Palette.line),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              'SDK Clients',
              style: theme.textTheme.titleMedium?.copyWith(
                color: Palette.ink,
                fontWeight: FontWeight.w800,
              ),
            ),
          ),
          if (isBusy) ...[
            Text(
              'Syncing',
              style: theme.textTheme.bodySmall?.copyWith(
                color: Palette.ink.withValues(alpha: 0.62),
                fontWeight: FontWeight.w700,
              ),
            ),
            SizedBox(width: 1.w),
          ],
          OutlinedButton.icon(
            onPressed: () => onRefresh(),
            style: OutlinedButton.styleFrom(
              foregroundColor: Palette.ink,
              side: BorderSide(color: Palette.line),
              padding: EdgeInsets.symmetric(horizontal: 1.4.w, vertical: 1.2.h),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(14),
              ),
            ),
            icon: const Icon(Icons.refresh, size: 18),
            label: const Text('Refresh'),
          ),
        ],
      ),
    );
  }
}

// ─── Table header row ─────────────────────────────────────────────────────────

class _ClientTableHeader extends StatelessWidget {
  const _ClientTableHeader();

  @override
  Widget build(BuildContext context) {
    final style = Theme.of(context).textTheme.labelLarge?.copyWith(
      color: Palette.ink.withValues(alpha: 0.72),
      fontWeight: FontWeight.w800,
      letterSpacing: 0.3,
    );

    return Container(
      padding: EdgeInsets.symmetric(vertical: 1.4.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(16),
        color: Palette.ink.withValues(alpha: 0.04),
      ),
      child: Row(
        children: [
          SizedBox(width: _accentStripWidth + _cellHPad),
          SizedBox(
            width: _idColWidth,
            child: Text('ID', style: style),
          ),
          SizedBox(width: _colGap),
          SizedBox(
            width: _nameColWidth,
            child: Text('Name', style: style),
          ),
          SizedBox(width: _colGap),
          SizedBox(
            width: _versionColWidth,
            child: Text('Version', style: style),
          ),
          SizedBox(width: _colGap),
          SizedBox(
            width: _registeredColWidth,
            child: Text('Registered', style: style),
          ),
          SizedBox(width: _colGap),
          SizedBox(width: _chevronColWidth),
          SizedBox(width: _cellHPad),
        ],
      ),
    );
  }
}

// ─── Table row (owns its own expand state) ────────────────────────────────────

class _ClientTableRow extends HookWidget {
  const _ClientTableRow({required this.client});

  final SdkClientEntry client;

  @override
  Widget build(BuildContext context) {
    final expanded = useState(false);
    final accent = _accentColor(client.pubkey);
    final theme = Theme.of(context);
    final muted = Palette.ink.withValues(alpha: 0.62);

    final name = client.info.name.isEmpty ? '—' : client.info.name;
    final version = client.info.version.isEmpty ? '—' : client.info.version;
    final registered = _formatDate(client.createdAt);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Collapsed row ──────────────────────────────────────────────────────
        GestureDetector(
          onTap: () => expanded.value = !expanded.value,
          child: Container(
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(18),
              color: accent.withValues(alpha: 0.10),
              border: Border.all(color: accent.withValues(alpha: 0.28)),
            ),
            child: IntrinsicHeight(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Container(
                    width: _accentStripWidth,
                    decoration: BoxDecoration(
                      color: accent,
                      borderRadius: const BorderRadius.horizontal(
                        left: Radius.circular(18),
                      ),
                    ),
                  ),
                  Expanded(
                    child: Padding(
                      padding: EdgeInsets.symmetric(
                        horizontal: _cellHPad,
                        vertical: 1.4.h,
                      ),
                      child: Row(
                        children: [
                          SizedBox(
                            width: _idColWidth,
                            child: Text(
                              '${client.id}',
                              style: theme.textTheme.bodyLarge?.copyWith(
                                color: Palette.ink,
                              ),
                            ),
                          ),
                          SizedBox(width: _colGap),
                          SizedBox(
                            width: _nameColWidth,
                            child: Text(
                              name,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: theme.textTheme.bodyMedium?.copyWith(
                                color: Palette.ink,
                              ),
                            ),
                          ),
                          SizedBox(width: _colGap),
                          SizedBox(
                            width: _versionColWidth,
                            child: Text(
                              version,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: theme.textTheme.bodySmall?.copyWith(
                                color: muted,
                              ),
                            ),
                          ),
                          SizedBox(width: _colGap),
                          SizedBox(
                            width: _registeredColWidth,
                            child: Text(
                              registered,
                              style: theme.textTheme.bodySmall?.copyWith(
                                color: muted,
                              ),
                            ),
                          ),
                          SizedBox(width: _colGap),
                          SizedBox(
                            width: _chevronColWidth,
                            child: Icon(
                              expanded.value
                                  ? Icons.expand_more
                                  : Icons.chevron_right,
                              color: muted,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
        // ── Expansion panel (AnimatedSize wraps only this section) ────────────
        AnimatedSize(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeInOut,
          child: expanded.value
              ? Container(
                  margin: EdgeInsets.only(top: 0.6.h),
                  padding: EdgeInsets.symmetric(
                    horizontal: 1.6.w,
                    vertical: 1.4.h,
                  ),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(14),
                    color: accent.withValues(alpha: 0.06),
                    border: Border(
                      left: BorderSide(color: accent, width: 0.4.w),
                    ),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        client.info.description.isEmpty
                            ? 'No description provided.'
                            : client.info.description,
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: muted,
                          height: 1.5,
                        ),
                      ),
                      SizedBox(height: 1.h),
                      Row(
                        children: [
                          Expanded(
                            child: Text(
                              _shortPubkey(client.pubkey),
                              style: theme.textTheme.bodySmall?.copyWith(
                                color: Palette.ink,
                                fontFamily: 'monospace',
                              ),
                            ),
                          ),
                          IconButton(
                            icon: const Icon(Icons.copy_rounded, size: 18),
                            color: muted,
                            onPressed: () async {
                              await Clipboard.setData(
                                ClipboardData(text: _fullPubkey(client.pubkey)),
                              );
                              if (!context.mounted) return;
                              ScaffoldMessenger.of(context).showSnackBar(
                                const SnackBar(
                                  content: Text('Public key copied.'),
                                  behavior: SnackBarBehavior.floating,
                                ),
                              );
                            },
                          ),
                          FilledButton.tonal(
                            onPressed: () {
                              context.router.push(
                                ClientDetailsRoute(clientId: client.id),
                              );
                            },
                            child: const Text('Manage access'),
                          ),
                        ],
                      ),
                    ],
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}

// ─── Table container ──────────────────────────────────────────────────────────

class _ClientTable extends StatelessWidget {
  const _ClientTable({required this.clients});

  final List<SdkClientEntry> clients;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return CreamFrame(
      padding: EdgeInsets.all(2.h),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final tableWidth = math.max(_tableMinWidth, constraints.maxWidth);

            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Registered clients',
                  style: theme.textTheme.titleLarge?.copyWith(
                    color: Palette.ink,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                SizedBox(height: 0.6.h),
                Text(
                  'Every entry here has authenticated with Arbiter at least once.',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: Palette.ink.withValues(alpha: 0.70),
                    height: 1.4,
                  ),
                ),
                SizedBox(height: 1.6.h),
                SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: SizedBox(
                    width: tableWidth,
                    child: Column(
                      children: [
                        const _ClientTableHeader(),
                        SizedBox(height: 1.h),
                        for (var i = 0; i < clients.length; i++)
                          Padding(
                            padding: EdgeInsets.only(
                              bottom: i == clients.length - 1 ? 0 : 1.h,
                            ),
                            child: _ClientTableRow(client: clients[i]),
                          ),
                      ],
                    ),
                  ),
                ),
              ],
            );
          },
        ),
    );
  }
}

// ─── Screen ───────────────────────────────────────────────────────────────────

@RoutePage()
class ClientsScreen extends HookConsumerWidget {
  const ClientsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final clientsAsync = ref.watch(sdkClientsProvider);
    final isConnected =
        ref.watch(connectionManagerProvider).asData?.value != null;

    void showMessage(String message) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
      );
    }

    Future<void> refresh() async {
      try {
        final future = ref.refresh(sdkClientsProvider.future);
        await future;
      } catch (error) {
        showMessage(_formatError(error));
      }
    }

    final clients = clientsAsync.asData?.value;

    final content = switch (clientsAsync) {
      AsyncLoading() when clients == null => const StatePanel(
        icon: Icons.hourglass_top,
        title: 'Loading clients',
        body: 'Pulling client registry from Arbiter.',
        busy: true,
      ),
      AsyncError(:final error) => StatePanel(
        icon: Icons.sync_problem,
        title: 'Client registry unavailable',
        body: _formatError(error),
        actionLabel: 'Retry',
        onAction: refresh,
      ),
      _ when !isConnected => StatePanel(
        icon: Icons.portable_wifi_off,
        title: 'No active server connection',
        body: 'Reconnect to Arbiter to list SDK clients.',
        actionLabel: 'Refresh',
        onAction: refresh,
      ),
      _ when clients != null && clients.isEmpty => StatePanel(
        icon: Icons.devices_other_outlined,
        title: 'No clients yet',
        body: 'SDK clients appear here once they register with Arbiter.',
        actionLabel: 'Refresh',
        onAction: refresh,
      ),
      _ => _ClientTable(clients: clients ?? const []),
    };

    return Scaffold(
      body: SafeArea(
        child: RefreshIndicator.adaptive(
          color: Palette.ink,
          backgroundColor: Colors.white,
          onRefresh: refresh,
          child: ListView(
            physics: const BouncingScrollPhysics(
              parent: AlwaysScrollableScrollPhysics(),
            ),
            padding: EdgeInsets.fromLTRB(2.4.w, 2.4.h, 2.4.w, 3.2.h),
            children: [
              _Header(isBusy: clientsAsync.isLoading, onRefresh: refresh),
              SizedBox(height: 1.8.h),
              content,
            ],
          ),
        ),
      ),
    );
  }
}
