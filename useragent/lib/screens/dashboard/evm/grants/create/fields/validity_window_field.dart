// lib/screens/dashboard/evm/grants/create/fields/validity_window_field.dart
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/date_time_field.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

class ValidityWindowField extends StatelessWidget {
  const ValidityWindowField({super.key});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: FormBuilderDateTimeField(
            name: 'validFrom',
            label: 'Valid from',
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: FormBuilderDateTimeField(
            name: 'validUntil',
            label: 'Valid until',
          ),
        ),
      ],
    );
  }
}
