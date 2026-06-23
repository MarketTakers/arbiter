import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';

/// A card-shaped frame with the cream background, rounded corners, and a
/// subtle border. Use [padding] for interior spacing and [margin] for exterior
/// spacing.
class CreamFrame extends StatelessWidget {
  const CreamFrame({
    super.key,
    required this.child,
    this.padding = EdgeInsets.zero,
    this.margin,
  });

  final Widget child;
  final EdgeInsetsGeometry padding;
  final EdgeInsetsGeometry? margin;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: margin,
      padding: padding,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: Palette.cream,
        border: Border.all(color: Palette.line),
      ),
      child: child,
    );
  }
}
