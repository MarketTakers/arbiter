import 'package:flutter/material.dart';

class WalletAccessSearchField extends StatelessWidget {
  const WalletAccessSearchField({
    super.key,
    required this.searchQuery,
    required this.onChanged,
  });

  final String searchQuery;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return TextFormField(
      initialValue: searchQuery,
      decoration: const InputDecoration(
        labelText: 'Search wallets',
        prefixIcon: Icon(Icons.search),
      ),
      onChanged: onChanged,
    );
  }
}
