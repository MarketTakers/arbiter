// lib/screens/dashboard/evm/grants/create/fields/chain_id_field.dart
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';

class ChainIdField extends StatelessWidget {
  const ChainIdField({super.key});

  @override
  Widget build(BuildContext context) {
    return FormBuilderTextField(
      name: 'chainId',
      initialValue: '1',
      keyboardType: TextInputType.number,
      decoration: const InputDecoration(
        labelText: 'Chain ID',
        hintText: '1',
        border: OutlineInputBorder(),
      ),
    );
  }
}
