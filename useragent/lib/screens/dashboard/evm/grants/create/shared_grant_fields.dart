// lib/screens/dashboard/evm/grants/create/shared_grant_fields.dart
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/client_picker_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/wallet_access_picker_field.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

class AuthorizationFields extends StatelessWidget {
  const AuthorizationFields({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const ClientPickerField(),
        SizedBox(height: 1.6.h),
        const WalletAccessPickerField(),
      ],
    );
  }
}
