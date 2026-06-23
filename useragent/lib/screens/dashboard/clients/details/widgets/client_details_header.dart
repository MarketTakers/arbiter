import 'package:flutter/material.dart';

class ClientDetailsHeader extends StatelessWidget {
  const ClientDetailsHeader({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Row(
      children: [
        BackButton(onPressed: () => Navigator.of(context).maybePop()),
        Expanded(
          child: Text(
            'Client Details',
            style: theme.textTheme.headlineSmall?.copyWith(
              fontWeight: FontWeight.w800,
            ),
          ),
        ),
      ],
    );
  }
}
