import 'package:arbiter/widgets/cream_frame.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

class StatePanel extends StatelessWidget {
  const StatePanel({
    super.key,
    required this.icon,
    required this.title,
    required this.body,
    this.actionLabel,
    this.onAction,
    this.busy = false,
  });

  final IconData icon;
  final String title;
  final String body;
  final String? actionLabel;
  final Future<void> Function()? onAction;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return CreamFrame(
      padding: EdgeInsets.all(2.8.h),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (busy)
            SizedBox(
              width: 2.8.h,
              height: 2.8.h,
              child: const CircularProgressIndicator(strokeWidth: 2.5),
            )
          else
            Icon(icon, size: 34, color: Palette.coral),
          SizedBox(height: 1.8.h),
          Text(
            title,
            style: theme.textTheme.headlineSmall?.copyWith(
              color: Palette.ink,
              fontWeight: FontWeight.w800,
            ),
          ),
          SizedBox(height: 1.h),
          Text(
            body,
            style: theme.textTheme.bodyLarge?.copyWith(
              color: Palette.ink.withValues(alpha: 0.72),
              height: 1.5,
            ),
          ),
          if (actionLabel != null && onAction != null) ...[
            SizedBox(height: 2.h),
            OutlinedButton.icon(
              onPressed: () => onAction!(),
              icon: const Icon(Icons.refresh),
              label: Text(actionLabel!),
            ),
          ],
        ],
      ),
    );
  }
}
