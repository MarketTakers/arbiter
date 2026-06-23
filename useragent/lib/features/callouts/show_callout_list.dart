import 'package:arbiter/features/callouts/active_callout.dart';
import 'package:arbiter/features/callouts/callout_manager.dart';
import 'package:arbiter/features/callouts/show_callout.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';
import 'package:timeago/timeago.dart' as timeago;

Future<void> showCalloutList(BuildContext context, WidgetRef ref) async {
  final selectedId = await showGeneralDialog<String>(
    context: context,
    barrierDismissible: true,
    barrierLabel: MaterialLocalizations.of(context).modalBarrierDismissLabel,
    barrierColor: Colors.transparent,
    transitionDuration: const Duration(milliseconds: 280),
    pageBuilder: (_, animation, __) =>
        _CalloutListOverlay(animation: animation),
  );

  if (selectedId != null && context.mounted) {
    await showCallout(context, ref, selectedId);
  }
}

class _CalloutListOverlay extends ConsumerWidget {
  const _CalloutListOverlay({required this.animation});

  final Animation<double> animation;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final callouts = ref.watch(calloutManagerProvider);

    final barrierAnim = CurvedAnimation(
      parent: animation,
      curve: const Interval(0, 0.3, curve: Curves.easeOut),
    );
    final panelAnim = CurvedAnimation(
      parent: animation,
      curve: const Interval(0.3, 1, curve: Curves.easeOutCubic),
    );

    return Material(
      type: MaterialType.transparency,
      child: Stack(
        children: [
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => Navigator.of(context).pop(),
              child: AnimatedBuilder(
                animation: barrierAnim,
                builder: (_, __) => ColoredBox(
                  color: Colors.black.withValues(
                    alpha: 0.35 * barrierAnim.value,
                  ),
                ),
              ),
            ),
          ),
          SafeArea(
            child: Align(
              alignment: Alignment.bottomCenter,
              child: Padding(
                padding: EdgeInsets.all(1.6.h),
                child: FadeTransition(
                  opacity: panelAnim,
                  child: SlideTransition(
                    position: Tween<Offset>(
                      begin: const Offset(0, 0.08),
                      end: Offset.zero,
                    ).animate(panelAnim),
                    child: GestureDetector(
                      onTap: () {},
                      child: _CalloutListPanel(callouts: callouts),
                    ),
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

class _CalloutListPanel extends StatelessWidget {
  const _CalloutListPanel({required this.callouts});

  final Map<String, ActiveCallout> callouts;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      width: double.infinity,
      constraints: BoxConstraints(maxHeight: 48.h),
      decoration: BoxDecoration(
        color: Palette.cream,
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: Palette.line),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: EdgeInsets.fromLTRB(2.h, 2.h, 2.h, 1.2.h),
            child: Text(
              'Notifications',
              style: theme.textTheme.titleMedium?.copyWith(
                color: Palette.ink,
                fontWeight: FontWeight.w800,
              ),
            ),
          ),
          if (callouts.isEmpty)
            Padding(
              padding: EdgeInsets.fromLTRB(2.h, 0, 2.h, 2.h),
              child: Text(
                'No pending notifications.',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: Palette.ink.withValues(alpha: 0.50),
                ),
              ),
            )
          else
            Flexible(
              child: SingleChildScrollView(
                padding: EdgeInsets.fromLTRB(1.2.h, 0, 1.2.h, 1.2.h),
                child: Column(
                  spacing: 0.5.h,
                  children: [
                    for (final entry in callouts.values)
                      _CalloutListEntry(
                        callout: entry,
                        onTap: () => Navigator.of(context).pop(entry.id),
                      ),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _CalloutListEntry extends StatelessWidget {
  const _CalloutListEntry({required this.callout, required this.onTap});

  final ActiveCallout callout;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return InkWell(
      borderRadius: BorderRadius.circular(16),
      onTap: onTap,
      child: Container(
        padding: EdgeInsets.symmetric(horizontal: 1.2.h, vertical: 1.2.h),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Palette.line),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 1.2.h,
          children: [
            if (callout.iconUrl != null)
              CircleAvatar(
                radius: 2.2.h,
                backgroundColor: Palette.line,
                backgroundImage: NetworkImage(callout.iconUrl!),
              ),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                spacing: 0.3.h,
                children: [
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Expanded(
                        child: Text(
                          callout.title,
                          style: theme.textTheme.bodyMedium?.copyWith(
                            color: Palette.ink,
                            fontWeight: FontWeight.w700,
                          ),
                        ),
                      ),
                      Text(
                        timeago.format(callout.addedAt),
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: Palette.ink.withValues(alpha: 0.45),
                        ),
                      ),
                    ],
                  ),
                  Text(
                    callout.description,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: Palette.ink.withValues(alpha: 0.65),
                      height: 1.4,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
