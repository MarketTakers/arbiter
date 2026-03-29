import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/widgets/cream_frame.dart';
import 'package:flutter/material.dart';

class ClientSummaryCard extends StatelessWidget {
  const ClientSummaryCard({super.key, required this.client});

  final SdkClientEntry client;

  @override
  Widget build(BuildContext context) {
    return CreamFrame(
      padding: const EdgeInsets.all(20),
      child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              client.info.name,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(client.info.description),
            const SizedBox(height: 16),
            Wrap(
              runSpacing: 8,
              spacing: 16,
              children: [
                _Fact(label: 'Client ID', value: '${client.id}'),
                _Fact(label: 'Version', value: client.info.version),
                _Fact(
                  label: 'Registered',
                  value: _formatDate(client.createdAt),
                ),
                _Fact(label: 'Pubkey', value: _shortPubkey(client.pubkey)),
              ],
            ),
          ],
        ),
    );
  }
}

class _Fact extends StatelessWidget {
  const _Fact({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: theme.textTheme.labelMedium),
        Text(value.isEmpty ? '—' : value, style: theme.textTheme.bodyMedium),
      ],
    );
  }
}

String _formatDate(int unixSecs) {
  final dt = DateTime.fromMillisecondsSinceEpoch(unixSecs * 1000).toLocal();
  return '${dt.year}-${dt.month.toString().padLeft(2, '0')}-${dt.day.toString().padLeft(2, '0')}';
}

String _shortPubkey(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  if (hex.length < 12) {
    return '0x$hex';
  }
  return '0x${hex.substring(0, 8)}...${hex.substring(hex.length - 4)}';
}
