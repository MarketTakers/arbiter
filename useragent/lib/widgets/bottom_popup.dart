import 'package:flutter/material.dart';

Future<T?> showBottomPopup<T>({
  required BuildContext context,
  required WidgetBuilder builder,
  bool barrierDismissible = true,
}) {
  return showGeneralDialog<T>(
    context: context,
    barrierDismissible: barrierDismissible,
    barrierLabel: MaterialLocalizations.of(context).modalBarrierDismissLabel,
    barrierColor: Colors.transparent,
    transitionDuration: const Duration(milliseconds: 320),
    pageBuilder: (dialogContext, animation, secondaryAnimation) {
      return _BottomPopupRoute(
        animation: animation,
        builder: builder,
        barrierDismissible: barrierDismissible,
      );
    },
  );
}

class _BottomPopupRoute extends StatelessWidget {
  const _BottomPopupRoute({
    required this.animation,
    required this.builder,
    required this.barrierDismissible,
  });

  final Animation<double> animation;
  final WidgetBuilder builder;
  final bool barrierDismissible;

  @override
  Widget build(BuildContext context) {
    final barrierAnimation = CurvedAnimation(
      parent: animation,
      curve: const Interval(0, 0.3125, curve: Curves.easeOut),
    );
    final popupAnimation = CurvedAnimation(
      parent: animation,
      curve: const Interval(0.3125, 1, curve: Curves.easeOutCubic),
    );

    return Material(
      type: MaterialType.transparency,
      child: Stack(
        children: [
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: barrierDismissible
                  ? () => Navigator.of(context).pop()
                  : null,
              child: AnimatedBuilder(
                animation: barrierAnimation,
                builder: (context, child) {
                  return ColoredBox(
                    color: Colors.black.withValues(
                      alpha: 0.35 * barrierAnimation.value,
                    ),
                  );
                },
              ),
            ),
          ),
          SafeArea(
            child: Align(
              alignment: Alignment.bottomCenter,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: FadeTransition(
                  opacity: popupAnimation,
                  child: SlideTransition(
                    position: Tween<Offset>(
                      begin: const Offset(0, 0.08),
                      end: Offset.zero,
                    ).animate(popupAnimation),
                    child: GestureDetector(
                      onTap: () {},
                      child: Builder(builder: builder),
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
