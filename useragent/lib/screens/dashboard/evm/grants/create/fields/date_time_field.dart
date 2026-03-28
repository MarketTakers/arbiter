// lib/screens/dashboard/evm/grants/create/fields/date_time_field.dart
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:sizer/sizer.dart';

/// A [FormBuilderField] that opens a date picker followed by a time picker.
/// Long-press clears the value.
class FormBuilderDateTimeField extends FormBuilderField<DateTime?> {
  final String label;

  FormBuilderDateTimeField({
    super.key,
    required super.name,
    required this.label,
    super.initialValue,
    super.onChanged,
    super.validator,
  }) : super(
          builder: (FormFieldState<DateTime?> field) {
            final value = field.value;
            return OutlinedButton(
              onPressed: () async {
                final ctx = field.context;
                final now = DateTime.now();
                final date = await showDatePicker(
                  context: ctx,
                  firstDate: DateTime(now.year - 5),
                  lastDate: DateTime(now.year + 10),
                  initialDate: value ?? now,
                );
                if (date == null) return;
                if (!ctx.mounted) return;
                final time = await showTimePicker(
                  context: ctx,
                  initialTime: TimeOfDay.fromDateTime(value ?? now),
                );
                if (time == null) return;
                field.didChange(DateTime(
                  date.year,
                  date.month,
                  date.day,
                  time.hour,
                  time.minute,
                ));
              },
              onLongPress: value == null ? null : () => field.didChange(null),
              child: Padding(
                padding: EdgeInsets.symmetric(vertical: 1.8.h),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(label),
                    SizedBox(height: 0.6.h),
                    Text(value?.toLocal().toString() ?? 'Not set'),
                  ],
                ),
              ),
            );
          },
        );
}
