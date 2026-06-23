import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

class PageHeader extends StatelessWidget {
  const PageHeader({
    super.key,
    required this.title,
    this.isBusy = false,
    this.busyLabel = 'Syncing',
    this.actions = const <Widget>[],
    this.padding,
    this.backgroundColor,
    this.borderColor,
  });

  final String title;
  final bool isBusy;
  final String busyLabel;
  final List<Widget> actions;
  final EdgeInsetsGeometry? padding;
  final Color? backgroundColor;
  final Color? borderColor;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding:
          padding ?? EdgeInsets.symmetric(horizontal: 1.6.w, vertical: 1.2.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(18),
        color: backgroundColor ?? Palette.cream,
        border: Border.all(color: borderColor ?? Palette.line),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              title,
              style: theme.textTheme.titleMedium?.copyWith(
                color: Palette.ink,
                fontWeight: FontWeight.w800,
              ),
            ),
          ),
          if (isBusy) ...[
            Text(
              busyLabel,
              style: theme.textTheme.bodySmall?.copyWith(
                color: Palette.ink.withValues(alpha: 0.62),
                fontWeight: FontWeight.w700,
              ),
            ),
            SizedBox(width: 1.w),
          ],
          ...actions,
        ],
      ),
    );
  }
}
