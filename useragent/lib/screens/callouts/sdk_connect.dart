import 'package:arbiter/proto/client.pb.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

class SdkConnectCallout extends StatelessWidget {
  const SdkConnectCallout({
    super.key,
    required this.pubkey,
    required this.clientInfo,
    this.onAccept,
    this.onDecline,
  });

  final String pubkey;
  final ClientInfo clientInfo;
  final VoidCallback? onAccept;
  final VoidCallback? onDecline;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    final name = clientInfo.hasName() && clientInfo.name.isNotEmpty
        ? clientInfo.name
        : _shortPubkey(pubkey);

    final hasDescription =
        clientInfo.hasDescription() && clientInfo.description.isNotEmpty;
    final hasVersion =
        clientInfo.hasVersion() && clientInfo.version.isNotEmpty;
    final showInfoCard = hasDescription || hasVersion;

    return Container(
      decoration: BoxDecoration(
        color: Palette.cream,
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: Palette.line),
      ),
      padding: EdgeInsets.all(2.4.h),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.center,
        spacing: 1.6.h,
        children: [
          // if (clientInfo.iconUrl != null)
          //   CircleAvatar(
          //     radius: 36,
          //     backgroundColor: Palette.line,
          //     backgroundImage: NetworkImage(iconUrl!),
          //   ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            spacing: 0.4.h,
            children: [
              Text(
                name,
                textAlign: TextAlign.center,
                style: theme.textTheme.titleLarge?.copyWith(
                  color: Palette.ink,
                  fontWeight: FontWeight.w800,
                ),
              ),
              Text(
                'is requesting a connection',
                textAlign: TextAlign.center,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: Palette.ink.withValues(alpha: 0.55),
                ),
              ),
            ],
          ),
          if (showInfoCard)
            Container(
              width: double.infinity,
              decoration: BoxDecoration(
                color: Palette.ink.withValues(alpha: 0.04),
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: Palette.line),
              ),
              padding: EdgeInsets.symmetric(
                horizontal: 1.6.w,
                vertical: 1.2.h,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                spacing: 0.6.h,
                children: [
                  if (hasDescription)
                    Text(
                      clientInfo.description,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: Palette.ink.withValues(alpha: 0.80),
                        height: 1.5,
                      ),
                    ),
                  if (hasVersion)
                    Text(
                      'v${clientInfo.version}',
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: Palette.ink.withValues(alpha: 0.50),
                      ),
                    ),
                ],
              ),
            ),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: onDecline,
                  style: OutlinedButton.styleFrom(
                    foregroundColor: Palette.coral,
                    side: BorderSide(
                      color: Palette.coral.withValues(alpha: 0.50),
                    ),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(14),
                    ),
                    padding: EdgeInsets.symmetric(vertical: 1.4.h),
                  ),
                  child: const Text('Decline'),
                ),
              ),
              Expanded(
                child: FilledButton(
                  onPressed: onAccept,
                  style: FilledButton.styleFrom(
                    backgroundColor: Palette.ink,
                    foregroundColor: Palette.cream,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(14),
                    ),
                    padding: EdgeInsets.symmetric(vertical: 1.4.h),
                  ),
                  child: const Text('Accept'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

String _shortPubkey(String base64Pubkey) {
  if (base64Pubkey.length < 12) return base64Pubkey;
  return '${base64Pubkey.substring(0, 8)}…${base64Pubkey.substring(base64Pubkey.length - 4)}';
}
