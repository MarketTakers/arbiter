// lib/screens/dashboard/evm/grants/create/screen.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/evm/evm_grants.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/ether_transfer_grant.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/token_transfer_grant.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/provider.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/chain_id_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/gas_fee_options_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/transaction_rate_limit_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/fields/validity_window_field.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/shared_grant_fields.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
import 'package:arbiter/theme/palette.dart';
import 'package:auto_route/auto_route.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:sizer/sizer.dart';

const _etherHandler = EtherTransferGrantHandler();
const _tokenHandler = TokenTransferGrantHandler();

GrantFormHandler _handlerFor(SpecificGrant_Grant type) => switch (type) {
  SpecificGrant_Grant.etherTransfer => _etherHandler,
  SpecificGrant_Grant.tokenTransfer => _tokenHandler,
  _ => throw ArgumentError('Unsupported grant type: $type'),
};

@RoutePage()
class CreateEvmGrantScreen extends HookConsumerWidget {
  const CreateEvmGrantScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final formKey = useMemoized(() => GlobalKey<FormBuilderState>());
    final createMutation = ref.watch(createEvmGrantMutation);
    final state = ref.watch(grantCreationProvider);
    final notifier = ref.read(grantCreationProvider.notifier);
    final handler = _handlerFor(state.grantType);

    Future<void> submit() async {
      if (!(formKey.currentState?.saveAndValidate() ?? false)) return;
      final formValues = formKey.currentState!.value;

      final accessId = formValues['walletAccessId'] as int?;
      if (accessId == null) {
        _showSnackBar(context, 'Select a client and wallet access.');
        return;
      }

      try {
        final specific = handler.buildSpecificGrant(formValues, ref);
        final sharedSettings = SharedSettings(
          walletAccessId: accessId,
          chainId: Int64.parseInt(
            (formValues['chainId'] as String? ?? '').trim(),
          ),
        );
        final validFrom = formValues['validFrom'] as DateTime?;
        final validUntil = formValues['validUntil'] as DateTime?;
        if (validFrom != null)
          sharedSettings.validFrom = toTimestamp(validFrom);
        if (validUntil != null) {
          sharedSettings.validUntil = toTimestamp(validUntil);
        }
        final gasBytes = optionalBigIntBytes(
          formValues['maxGasFeePerGas'] as String? ?? '',
        );
        if (gasBytes != null) sharedSettings.maxGasFeePerGas = gasBytes;
        final priorityBytes = optionalBigIntBytes(
          formValues['maxPriorityFeePerGas'] as String? ?? '',
        );
        if (priorityBytes != null) {
          sharedSettings.maxPriorityFeePerGas = priorityBytes;
        }
        final rateLimit = buildRateLimit(
          formValues['txCount'] as String? ?? '',
          formValues['txWindow'] as String? ?? '',
        );
        if (rateLimit != null) sharedSettings.rateLimit = rateLimit;

        await executeCreateEvmGrant(
          ref,
          sharedSettings: sharedSettings,
          specific: specific,
        );
        if (!context.mounted) return;
        context.router.pop();
      } catch (error) {
        if (!context.mounted) return;
        _showSnackBar(context, _formatError(error));
      }
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Create EVM Grant')),
      body: SafeArea(
        child: FormBuilder(
          key: formKey,
          child: ListView(
            padding: EdgeInsets.fromLTRB(2.4.w, 2.h, 2.4.w, 3.2.h),
            children: [
              const _IntroCard(),
              SizedBox(height: 1.8.h),
              const _Section(
                title: 'Authorization',
                tooltip:
                    'Select which SDK client receives this grant and '
                    'which of its wallet accesses it applies to.',
                child: AuthorizationFields(),
              ),
              SizedBox(height: 1.8.h),
              IntrinsicHeight(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Expanded(
                      child: _Section(
                        title: 'Chain',
                        tooltip:
                            'Restrict this grant to a specific EVM chain ID. '
                            'Leave empty to allow any chain.',
                        optional: true,
                        child: ChainIdField(),
                      ),
                    ),
                    SizedBox(width: 1.8.w),
                    const Expanded(
                      child: _Section(
                        title: 'Timing',
                        tooltip:
                            'Set an optional validity window. '
                            'Signing requests outside this period will be rejected.',
                        optional: true,
                        child: ValidityWindowField(),
                      ),
                    ),
                  ],
                ),
              ),
              SizedBox(height: 1.8.h),
              IntrinsicHeight(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Expanded(
                      child: _Section(
                        title: 'Gas limits',
                        tooltip:
                            'Cap the gas fees this grant may authorize. '
                            'Transactions exceeding these values will be rejected.',
                        optional: true,
                        child: GasFeeOptionsField(),
                      ),
                    ),
                    SizedBox(width: 1.8.w),
                    const Expanded(
                      child: _Section(
                        title: 'Transaction limits',
                        tooltip:
                            'Limit how many transactions can be signed '
                            'within a rolling time window.',
                        optional: true,
                        child: TransactionRateLimitField(),
                      ),
                    ),
                  ],
                ),
              ),
              SizedBox(height: 1.8.h),
              _GrantTypeSelector(
                value: state.grantType,
                onChanged: notifier.setGrantType,
              ),
              SizedBox(height: 1.8.h),
              _Section(
                title: 'Grant-specific options',
                tooltip:
                    'Rules specific to the selected transfer type. '
                    'Switch between Ether and token above to change these fields.',
                child: handler.buildForm(context, ref),
              ),
              SizedBox(height: 2.2.h),
              Align(
                alignment: Alignment.centerRight,
                child: FilledButton.icon(
                  onPressed: createMutation is MutationPending ? null : submit,
                  icon: createMutation is MutationPending
                      ? SizedBox(
                          width: 1.8.h,
                          height: 1.8.h,
                          child: const CircularProgressIndicator(
                            strokeWidth: 2.2,
                          ),
                        )
                      : const Icon(Icons.check_rounded),
                  label: Text(
                    createMutation is MutationPending
                        ? 'Creating...'
                        : 'Create grant',
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

class _IntroCard extends StatelessWidget {
  const _IntroCard();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(2.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        gradient: const LinearGradient(
          colors: [Palette.introGradientStart, Palette.introGradientEnd],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        border: Border.all(color: Palette.cardBorder),
      ),
      child: Text(
        'Pick a client, then select one of the wallet accesses already granted '
        'to it. Compose shared constraints once, then switch between Ether and '
        'token transfer rules.',
        style: Theme.of(context).textTheme.bodyLarge?.copyWith(height: 1.5),
      ),
    );
  }
}

class _Section extends StatelessWidget {
  const _Section({
    required this.title,
    required this.tooltip,
    required this.child,
    this.optional = false,
  });

  final String title;
  final String tooltip;
  final Widget child;
  final bool optional;

  @override
  Widget build(BuildContext context) {
    final subtleColor = Theme.of(context).colorScheme.outline;
    return Container(
      padding: EdgeInsets.all(2.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: Colors.white,
        border: Border.all(color: Palette.cardBorder),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                title,
                style: Theme.of(
                  context,
                ).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w800),
              ),
              SizedBox(width: 0.4.w),
              Tooltip(
                message: tooltip,
                child: Icon(
                  Icons.info_outline_rounded,
                  size: 16,
                  color: subtleColor,
                ),
              ),
              if (optional) ...[
                SizedBox(width: 0.6.w),
                Text(
                  '(optional)',
                  style: Theme.of(
                    context,
                  ).textTheme.bodySmall?.copyWith(color: subtleColor),
                ),
              ],
            ],
          ),
          SizedBox(height: 1.4.h),
          child,
        ],
      ),
    );
  }
}

class _GrantTypeSelector extends StatelessWidget {
  const _GrantTypeSelector({required this.value, required this.onChanged});

  final SpecificGrant_Grant value;
  final ValueChanged<SpecificGrant_Grant> onChanged;

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<SpecificGrant_Grant>(
      segments: const [
        ButtonSegment(
          value: SpecificGrant_Grant.etherTransfer,
          label: Text('Ether'),
          icon: Icon(Icons.bolt_rounded),
        ),
        ButtonSegment(
          value: SpecificGrant_Grant.tokenTransfer,
          label: Text('Token'),
          icon: Icon(Icons.token_rounded),
        ),
      ],
      selected: {value},
      onSelectionChanged: (selection) => onChanged(selection.first),
    );
  }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

void _showSnackBar(BuildContext context, String message) {
  if (!context.mounted) return;
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
  );
}

String _formatError(Object error) {
  final text = error.toString();
  return text.startsWith('Exception: ')
      ? text.substring('Exception: '.length)
      : text;
}
