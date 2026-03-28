// lib/screens/dashboard/evm/grants/create/shared_grant_fields.dart
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/chain_id_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/client_picker_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/gas_fee_options_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/transaction_rate_limit_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/validity_window_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/wallet_access_picker_field.dart';
import 'package:flutter/material.dart';
import 'package:sizer/sizer.dart';

/// All shared grant fields in a single vertical layout.
///
/// Every [FormBuilderField] descendant auto-registers with the nearest
/// [FormBuilder] ancestor via [BuildContext] — no controllers passed.
class SharedGrantFields extends StatelessWidget {
  const SharedGrantFields({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const ClientPickerField(),
        SizedBox(height: 1.6.h),
        const WalletAccessPickerField(),
        SizedBox(height: 1.6.h),
        const ChainIdField(),
        SizedBox(height: 1.6.h),
        const ValidityWindowField(),
        SizedBox(height: 1.6.h),
        const GasFeeOptionsField(),
        SizedBox(height: 1.6.h),
        const TransactionRateLimitField(),
      ],
    );
  }
}
