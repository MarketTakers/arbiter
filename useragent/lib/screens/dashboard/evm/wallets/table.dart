import 'dart:math' as math;
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

double get _accentStripWidth => 0.8.w;
double get _cellHorizontalPadding => 1.8.w;
double get _walletColumnWidth => 18.w;
double get _columnGap => 1.8.w;
double get _tableMinWidth => 72.w;

String _hexAddress(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '0x$hex';
}

Color _accentColor(List<int> bytes) {
  final seed = bytes.fold<int>(0, (value, byte) => value + byte);
  final hue = (seed * 17) % 360;
  return HSLColor.fromAHSL(1, hue.toDouble(), 0.68, 0.54).toColor();
}

class WalletTable extends StatelessWidget {
  const WalletTable({super.key, required this.wallets});

  final List<WalletEntry> wallets;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: Palette.cream.withValues(alpha: 0.92),
        border: Border.all(color: Palette.line),
      ),
      child: Padding(
        padding: EdgeInsets.all(2.h),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final tableWidth = math.max(_tableMinWidth, constraints.maxWidth);

            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Managed wallets',
                  style: theme.textTheme.titleLarge?.copyWith(
                    color: Palette.ink,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                SizedBox(height: 0.6.h),
                Text(
                  'Every address here is generated and held by Arbiter.',
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
                        const _WalletTableHeader(),
                        SizedBox(height: 1.h),
                        for (var i = 0; i < wallets.length; i++)
                          Padding(
                            padding: EdgeInsets.only(
                              bottom: i == wallets.length - 1 ? 0 : 1.h,
                            ),
                            child: _WalletTableRow(
                              wallet: wallets[i],
                              index: i,
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

class _WalletTableHeader extends StatelessWidget {
  const _WalletTableHeader();

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
          SizedBox(width: _accentStripWidth + _cellHorizontalPadding),
          SizedBox(
            width: _walletColumnWidth,
            child: Text('Wallet', style: style),
          ),
          SizedBox(width: _columnGap),
          Expanded(child: Text('Address', style: style)),
          SizedBox(width: _cellHorizontalPadding),
        ],
      ),
    );
  }
}

class _WalletTableRow extends StatelessWidget {
  const _WalletTableRow({required this.wallet, required this.index});

  final WalletEntry wallet;
  final int index;

  @override
  Widget build(BuildContext context) {
    final accent = _accentColor(wallet.address);
    final address = _hexAddress(wallet.address);
    final rowHeight = 5.h;
    final walletStyle = Theme.of(
      context,
    ).textTheme.bodyLarge?.copyWith(color: Palette.ink);
    final addressStyle = Theme.of(
      context,
    ).textTheme.bodyMedium?.copyWith(color: Palette.ink);

    return Container(
      height: rowHeight,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(18),
        color: accent.withValues(alpha: 0.10),
        border: Border.all(color: accent.withValues(alpha: 0.28)),
      ),
      child: Row(
        children: [
          Container(
            width: _accentStripWidth,
            height: rowHeight,
            decoration: BoxDecoration(
              color: accent,
              borderRadius: const BorderRadius.horizontal(
                left: Radius.circular(18),
              ),
            ),
          ),
          Expanded(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: _cellHorizontalPadding),
              child: Row(
                children: [
                  SizedBox(
                    width: _walletColumnWidth,
                    child: Row(
                      children: [
                        Container(
                          width: 1.2.h,
                          height: 1.2.h,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: accent,
                          ),
                        ),
                        SizedBox(width: 1.w),
                        Text(
                          'Wallet ${(index + 1).toString().padLeft(2, '0')}',
                          style: walletStyle,
                        ),
                      ],
                    ),
                  ),
                  SizedBox(width: _columnGap),
                  Expanded(
                    child: Text(
                      address,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: addressStyle,
                    ),
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
