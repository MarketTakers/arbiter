// lib/screens/dashboard/evm/grants/create/fields/gas_fee_options_field.dart
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:sizer/sizer.dart';

class GasFeeOptionsField extends StatelessWidget {
  const GasFeeOptionsField({super.key});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: FormBuilderTextField(
            name: 'maxGasFeePerGas',
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Max gas fee / gas',
              hintText: '1000000000',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: FormBuilderTextField(
            name: 'maxPriorityFeePerGas',
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Max priority fee / gas',
              hintText: '100000000',
              border: OutlineInputBorder(),
            ),
          ),
        ),
      ],
    );
  }
}
