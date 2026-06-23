// lib/screens/dashboard/evm/grants/create/fields/transaction_rate_limit_field.dart
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:sizer/sizer.dart';

class TransactionRateLimitField extends StatelessWidget {
  const TransactionRateLimitField({super.key});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: FormBuilderTextField(
            name: 'txCount',
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Tx count limit',
              hintText: '10',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: FormBuilderTextField(
            name: 'txWindow',
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Window (seconds)',
              hintText: '3600',
              border: OutlineInputBorder(),
            ),
          ),
        ),
      ],
    );
  }
}
