import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/evm.dart';
import 'package:arbiter/providers/evm_grants.dart';
import 'package:auto_route/auto_route.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:hooks_riverpod/experimental/mutation.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class CreateEvmGrantScreen extends HookConsumerWidget {
  const CreateEvmGrantScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final wallets = ref.watch(evmProvider).asData?.value ?? const <WalletEntry>[];
    final createMutation = ref.watch(createEvmGrantMutation);

    final selectedWalletIndex = useState<int?>(wallets.isEmpty ? null : 0);
    final clientIdController = useTextEditingController();
    final chainIdController = useTextEditingController(text: '1');
    final gasFeeController = useTextEditingController();
    final priorityFeeController = useTextEditingController();
    final txCountController = useTextEditingController();
    final txWindowController = useTextEditingController();
    final recipientsController = useTextEditingController();
    final etherVolumeController = useTextEditingController();
    final etherVolumeWindowController = useTextEditingController();
    final tokenContractController = useTextEditingController();
    final tokenTargetController = useTextEditingController();
    final validFrom = useState<DateTime?>(null);
    final validUntil = useState<DateTime?>(null);
    final grantType = useState<SpecificGrant_Grant>(
      SpecificGrant_Grant.etherTransfer,
    );
    final tokenVolumeLimits = useState<List<_VolumeLimitValue>>([
      const _VolumeLimitValue(),
    ]);

    Future<void> submit() async {
      final selectedWallet = selectedWalletIndex.value;
      if (selectedWallet == null) {
        _showCreateMessage(context, 'At least one wallet is required.');
        return;
      }

      try {
        final clientId = int.parse(clientIdController.text.trim());
        final chainId = Int64.parseInt(chainIdController.text.trim());
        final rateLimit = _buildRateLimit(
          txCountController.text,
          txWindowController.text,
        );
        final specific = switch (grantType.value) {
          SpecificGrant_Grant.etherTransfer => SpecificGrant(
            etherTransfer: EtherTransferSettings(
              targets: _parseAddresses(recipientsController.text),
              limit: _buildVolumeLimit(
                etherVolumeController.text,
                etherVolumeWindowController.text,
              ),
            ),
          ),
          SpecificGrant_Grant.tokenTransfer => SpecificGrant(
            tokenTransfer: TokenTransferSettings(
              tokenContract: _parseHexAddress(tokenContractController.text),
              target: tokenTargetController.text.trim().isEmpty
                  ? null
                  : _parseHexAddress(tokenTargetController.text),
              volumeLimits: tokenVolumeLimits.value
                  .where((item) => item.amount.trim().isNotEmpty)
                  .map(
                    (item) => VolumeRateLimit(
                      maxVolume: _parseBigIntBytes(item.amount),
                      windowSecs: Int64.parseInt(item.windowSeconds),
                    ),
                  )
                  .toList(),
            ),
          ),
          _ => throw Exception('Unsupported grant type.'),
        };

        await executeCreateEvmGrant(
          ref,
          clientId: clientId,
          walletId: selectedWallet + 1,
          chainId: chainId,
          validFrom: validFrom.value,
          validUntil: validUntil.value,
          maxGasFeePerGas: _optionalBigIntBytes(gasFeeController.text),
          maxPriorityFeePerGas: _optionalBigIntBytes(priorityFeeController.text),
          rateLimit: rateLimit,
          specific: specific,
        );
        if (!context.mounted) {
          return;
        }
        context.router.pop();
      } catch (error) {
        if (!context.mounted) {
          return;
        }
        _showCreateMessage(context, _formatCreateError(error));
      }
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Create EVM Grant')),
      body: SafeArea(
        child: ListView(
          padding: EdgeInsets.fromLTRB(2.4.w, 2.h, 2.4.w, 3.2.h),
          children: [
            _CreateIntroCard(walletCount: wallets.length),
            SizedBox(height: 1.8.h),
            _CreateSection(
              title: 'Shared grant options',
              children: [
                _WalletPickerField(
                  wallets: wallets,
                  selectedIndex: selectedWalletIndex.value,
                  onChanged: (value) => selectedWalletIndex.value = value,
                ),
                _NumberInputField(
                  controller: clientIdController,
                  label: 'Client ID',
                  hint: '42',
                  helper:
                      'Manual for now. The app does not yet expose a client picker.',
                ),
                _NumberInputField(
                  controller: chainIdController,
                  label: 'Chain ID',
                  hint: '1',
                ),
                _ValidityWindowField(
                  validFrom: validFrom.value,
                  validUntil: validUntil.value,
                  onValidFromChanged: (value) => validFrom.value = value,
                  onValidUntilChanged: (value) => validUntil.value = value,
                ),
                _GasFeeOptionsField(
                  gasFeeController: gasFeeController,
                  priorityFeeController: priorityFeeController,
                ),
                _TransactionRateLimitField(
                  txCountController: txCountController,
                  txWindowController: txWindowController,
                ),
              ],
            ),
            SizedBox(height: 1.8.h),
            _GrantTypeSelector(
              value: grantType.value,
              onChanged: (value) => grantType.value = value,
            ),
            SizedBox(height: 1.8.h),
            _CreateSection(
              title: 'Grant-specific options',
              children: [
                if (grantType.value == SpecificGrant_Grant.etherTransfer) ...[
                  _EtherTargetsField(controller: recipientsController),
                  _VolumeLimitField(
                    amountController: etherVolumeController,
                    windowController: etherVolumeWindowController,
                    title: 'Ether volume limit',
                  ),
                ] else ...[
                  _TokenContractField(controller: tokenContractController),
                  _TokenRecipientField(controller: tokenTargetController),
                  _TokenVolumeLimitsField(
                    values: tokenVolumeLimits.value,
                    onChanged: (values) => tokenVolumeLimits.value = values,
                  ),
                ],
              ],
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
                        child: const CircularProgressIndicator(strokeWidth: 2.2),
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
    );
  }
}

class _CreateIntroCard extends StatelessWidget {
  const _CreateIntroCard({required this.walletCount});

  final int walletCount;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(2.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        gradient: const LinearGradient(
          colors: [Color(0xFFF7F9FC), Color(0xFFFDF5EA)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        border: Border.all(color: const Color(0x1A17324A)),
      ),
      child: Text(
        'Compose shared constraints once, then switch between Ether and token transfer rules. $walletCount wallet${walletCount == 1 ? '' : 's'} currently available.',
        style: Theme.of(context).textTheme.bodyLarge?.copyWith(height: 1.5),
      ),
    );
  }
}

class _CreateSection extends StatelessWidget {
  const _CreateSection({required this.title, required this.children});

  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(2.h),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(24),
        color: Colors.white,
        border: Border.all(color: const Color(0x1A17324A)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.w800,
            ),
          ),
          SizedBox(height: 1.4.h),
          ...children.map(
            (child) => Padding(
              padding: EdgeInsets.only(bottom: 1.6.h),
              child: child,
            ),
          ),
        ],
      ),
    );
  }
}

class _WalletPickerField extends StatelessWidget {
  const _WalletPickerField({
    required this.wallets,
    required this.selectedIndex,
    required this.onChanged,
  });

  final List<WalletEntry> wallets;
  final int? selectedIndex;
  final ValueChanged<int?> onChanged;

  @override
  Widget build(BuildContext context) {
    return DropdownButtonFormField<int>(
      initialValue: selectedIndex,
      decoration: const InputDecoration(
        labelText: 'Wallet',
        helperText:
            'Uses the current wallet order. The API still does not expose stable wallet IDs directly.',
        border: OutlineInputBorder(),
      ),
      items: [
        for (var i = 0; i < wallets.length; i++)
          DropdownMenuItem(
            value: i,
            child: Text(
              'Wallet ${(i + 1).toString().padLeft(2, '0')} · ${_shortAddress(wallets[i].address)}',
            ),
          ),
      ],
      onChanged: wallets.isEmpty ? null : onChanged,
    );
  }
}

class _NumberInputField extends StatelessWidget {
  const _NumberInputField({
    required this.controller,
    required this.label,
    required this.hint,
    this.helper,
  });

  final TextEditingController controller;
  final String label;
  final String hint;
  final String? helper;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      keyboardType: TextInputType.number,
      decoration: InputDecoration(
        labelText: label,
        hintText: hint,
        helperText: helper,
        border: const OutlineInputBorder(),
      ),
    );
  }
}

class _ValidityWindowField extends StatelessWidget {
  const _ValidityWindowField({
    required this.validFrom,
    required this.validUntil,
    required this.onValidFromChanged,
    required this.onValidUntilChanged,
  });

  final DateTime? validFrom;
  final DateTime? validUntil;
  final ValueChanged<DateTime?> onValidFromChanged;
  final ValueChanged<DateTime?> onValidUntilChanged;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: _DateButtonField(
            label: 'Valid from',
            value: validFrom,
            onChanged: onValidFromChanged,
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: _DateButtonField(
            label: 'Valid until',
            value: validUntil,
            onChanged: onValidUntilChanged,
          ),
        ),
      ],
    );
  }
}

class _DateButtonField extends StatelessWidget {
  const _DateButtonField({
    required this.label,
    required this.value,
    required this.onChanged,
  });

  final String label;
  final DateTime? value;
  final ValueChanged<DateTime?> onChanged;

  @override
  Widget build(BuildContext context) {
    return OutlinedButton(
      onPressed: () async {
        final now = DateTime.now();
        final date = await showDatePicker(
          context: context,
          firstDate: DateTime(now.year - 5),
          lastDate: DateTime(now.year + 10),
          initialDate: value ?? now,
        );
        if (date == null || !context.mounted) {
          return;
        }
        final time = await showTimePicker(
          context: context,
          initialTime: TimeOfDay.fromDateTime(value ?? now),
        );
        if (time == null) {
          return;
        }
        onChanged(
          DateTime(date.year, date.month, date.day, time.hour, time.minute),
        );
      },
      onLongPress: value == null ? null : () => onChanged(null),
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
  }
}

class _GasFeeOptionsField extends StatelessWidget {
  const _GasFeeOptionsField({
    required this.gasFeeController,
    required this.priorityFeeController,
  });

  final TextEditingController gasFeeController;
  final TextEditingController priorityFeeController;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: _NumberInputField(
            controller: gasFeeController,
            label: 'Max gas fee / gas',
            hint: '1000000000',
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: _NumberInputField(
            controller: priorityFeeController,
            label: 'Max priority fee / gas',
            hint: '100000000',
          ),
        ),
      ],
    );
  }
}

class _TransactionRateLimitField extends StatelessWidget {
  const _TransactionRateLimitField({
    required this.txCountController,
    required this.txWindowController,
  });

  final TextEditingController txCountController;
  final TextEditingController txWindowController;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: _NumberInputField(
            controller: txCountController,
            label: 'Tx count limit',
            hint: '10',
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: _NumberInputField(
            controller: txWindowController,
            label: 'Window (seconds)',
            hint: '3600',
          ),
        ),
      ],
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

class _EtherTargetsField extends StatelessWidget {
  const _EtherTargetsField({required this.controller});

  final TextEditingController controller;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      minLines: 3,
      maxLines: 6,
      decoration: const InputDecoration(
        labelText: 'Ether recipients',
        hintText: 'One 0x address per line. Leave empty for unrestricted targets.',
        border: OutlineInputBorder(),
      ),
    );
  }
}

class _VolumeLimitField extends StatelessWidget {
  const _VolumeLimitField({
    required this.amountController,
    required this.windowController,
    required this.title,
  });

  final TextEditingController amountController;
  final TextEditingController windowController;
  final String title;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.labelLarge?.copyWith(
            fontWeight: FontWeight.w800,
          ),
        ),
        SizedBox(height: 0.8.h),
        Row(
          children: [
            Expanded(
              child: _NumberInputField(
                controller: amountController,
                label: 'Max volume',
                hint: '1000000000000000000',
              ),
            ),
            SizedBox(width: 1.w),
            Expanded(
              child: _NumberInputField(
                controller: windowController,
                label: 'Window (seconds)',
                hint: '86400',
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _TokenContractField extends StatelessWidget {
  const _TokenContractField({required this.controller});

  final TextEditingController controller;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      decoration: const InputDecoration(
        labelText: 'Token contract',
        hintText: '0x...',
        border: OutlineInputBorder(),
      ),
    );
  }
}

class _TokenRecipientField extends StatelessWidget {
  const _TokenRecipientField({required this.controller});

  final TextEditingController controller;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      decoration: const InputDecoration(
        labelText: 'Token recipient',
        hintText: '0x... or leave empty for any recipient',
        border: OutlineInputBorder(),
      ),
    );
  }
}

class _TokenVolumeLimitsField extends StatelessWidget {
  const _TokenVolumeLimitsField({
    required this.values,
    required this.onChanged,
  });

  final List<_VolumeLimitValue> values;
  final ValueChanged<List<_VolumeLimitValue>> onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                'Token volume limits',
                style: Theme.of(context).textTheme.labelLarge?.copyWith(
                  fontWeight: FontWeight.w800,
                ),
              ),
            ),
            TextButton.icon(
              onPressed: () =>
                  onChanged([...values, const _VolumeLimitValue()]),
              icon: const Icon(Icons.add_rounded),
              label: const Text('Add'),
            ),
          ],
        ),
        SizedBox(height: 0.8.h),
        for (var i = 0; i < values.length; i++)
          Padding(
            padding: EdgeInsets.only(bottom: 1.h),
            child: _TokenVolumeLimitRow(
              value: values[i],
              onChanged: (next) {
                final updated = [...values];
                updated[i] = next;
                onChanged(updated);
              },
              onRemove: values.length == 1
                  ? null
                  : () {
                      final updated = [...values]..removeAt(i);
                      onChanged(updated);
                    },
            ),
          ),
      ],
    );
  }
}

class _TokenVolumeLimitRow extends StatelessWidget {
  const _TokenVolumeLimitRow({
    required this.value,
    required this.onChanged,
    required this.onRemove,
  });

  final _VolumeLimitValue value;
  final ValueChanged<_VolumeLimitValue> onChanged;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final amountController = TextEditingController(text: value.amount);
    final windowController = TextEditingController(text: value.windowSeconds);

    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: amountController,
            onChanged: (next) =>
                onChanged(value.copyWith(amount: next)),
            decoration: const InputDecoration(
              labelText: 'Max volume',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        SizedBox(width: 1.w),
        Expanded(
          child: TextField(
            controller: windowController,
            onChanged: (next) =>
                onChanged(value.copyWith(windowSeconds: next)),
            decoration: const InputDecoration(
              labelText: 'Window (seconds)',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        SizedBox(width: 0.4.w),
        IconButton(
          onPressed: onRemove,
          icon: const Icon(Icons.remove_circle_outline_rounded),
        ),
      ],
    );
  }
}

class _VolumeLimitValue {
  const _VolumeLimitValue({this.amount = '', this.windowSeconds = ''});

  final String amount;
  final String windowSeconds;

  _VolumeLimitValue copyWith({String? amount, String? windowSeconds}) {
    return _VolumeLimitValue(
      amount: amount ?? this.amount,
      windowSeconds: windowSeconds ?? this.windowSeconds,
    );
  }
}

TransactionRateLimit? _buildRateLimit(String countText, String windowText) {
  if (countText.trim().isEmpty || windowText.trim().isEmpty) {
    return null;
  }
  return TransactionRateLimit(
    count: int.parse(countText.trim()),
    windowSecs: Int64.parseInt(windowText.trim()),
  );
}

VolumeRateLimit? _buildVolumeLimit(String amountText, String windowText) {
  if (amountText.trim().isEmpty || windowText.trim().isEmpty) {
    return null;
  }
  return VolumeRateLimit(
    maxVolume: _parseBigIntBytes(amountText),
    windowSecs: Int64.parseInt(windowText.trim()),
  );
}

List<int>? _optionalBigIntBytes(String value) {
  if (value.trim().isEmpty) {
    return null;
  }
  return _parseBigIntBytes(value);
}

List<int> _parseBigIntBytes(String value) {
  final number = BigInt.parse(value.trim());
  if (number < BigInt.zero) {
    throw Exception('Numeric values must be positive.');
  }
  if (number == BigInt.zero) {
    return [0];
  }

  var remaining = number;
  final bytes = <int>[];
  while (remaining > BigInt.zero) {
    bytes.insert(0, (remaining & BigInt.from(0xff)).toInt());
    remaining >>= 8;
  }
  return bytes;
}

List<List<int>> _parseAddresses(String input) {
  final parts = input
      .split(RegExp(r'[\n,]'))
      .map((part) => part.trim())
      .where((part) => part.isNotEmpty);
  return parts.map(_parseHexAddress).toList();
}

List<int> _parseHexAddress(String value) {
  final normalized = value.trim().replaceFirst(RegExp(r'^0x'), '');
  if (normalized.length != 40) {
    throw Exception('Expected a 20-byte hex address.');
  }
  return [
    for (var i = 0; i < normalized.length; i += 2)
      int.parse(normalized.substring(i, i + 2), radix: 16),
  ];
}

String _shortAddress(List<int> bytes) {
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '0x${hex.substring(0, 6)}...${hex.substring(hex.length - 4)}';
}

void _showCreateMessage(BuildContext context, String message) {
  if (!context.mounted) {
    return;
  }
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
  );
}

String _formatCreateError(Object error) {
  final text = error.toString();
  if (text.startsWith('Exception: ')) {
    return text.substring('Exception: '.length);
  }
  return text;
}
