import 'package:arbiter/features/callouts/callout_event.dart';
import 'package:arbiter/features/callouts/callout_manager.dart';
import 'package:arbiter/screens/callouts/sdk_connect.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

Future<void> showCallout(BuildContext context, WidgetRef ref, String id) async {
  final data = ref.read(calloutManagerProvider)[id]?.data;
  if (data == null) return;

  await showGeneralDialog(
    context: context,
    barrierDismissible: false,
    barrierLabel: MaterialLocalizations.of(context).modalBarrierDismissLabel,
    barrierColor: Colors.transparent,
    transitionDuration: const Duration(milliseconds: 320),
    pageBuilder: (_, animation, _) => _CalloutOverlay(
      id: id,
      data: data,
      animation: animation,
    ),
  );
}

class _CalloutOverlay extends ConsumerWidget {
  const _CalloutOverlay({
    required this.id,
    required this.data,
    required this.animation,
  });

  final String id;
  final CalloutData data;
  final Animation<double> animation;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(
      calloutManagerProvider.select((map) => map.containsKey(id)),
      (wasPresent, isPresent) {
        if (wasPresent == true && !isPresent && context.mounted) {
          Navigator.of(context).pop();
        }
      },
    );

    final content = switch (data) {
      ConnectApprovalData(:final pubkey, :final clientInfo) => SdkConnectCallout(
        pubkey: pubkey,
        clientInfo: clientInfo,
        onAccept: () => ref.read(calloutManagerProvider.notifier).sendDecision(id, true),
        onDecline: () => ref.read(calloutManagerProvider.notifier).sendDecision(id, false),
      ),
    };

    final barrierAnim = CurvedAnimation(
      parent: animation,
      curve: const Interval(0, 0.3125, curve: Curves.easeOut),
    );
    final popupAnim = CurvedAnimation(
      parent: animation,
      curve: const Interval(0.3125, 1, curve: Curves.easeOutCubic),
    );

    return Material(
      type: MaterialType.transparency,
      child: Stack(
        children: [
          Positioned.fill(
            child: AnimatedBuilder(
              animation: barrierAnim,
              builder: (_, __) => ColoredBox(
                color: Colors.black.withValues(alpha: 0.35 * barrierAnim.value),
              ),
            ),
          ),
          SafeArea(
            child: Align(
              alignment: Alignment.bottomCenter,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: FadeTransition(
                  opacity: popupAnim,
                  child: SlideTransition(
                    position: Tween<Offset>(
                      begin: const Offset(0, 0.08),
                      end: Offset.zero,
                    ).animate(popupAnim),
                    child: content,
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
