// lib/screens/dashboard/evm/grants/create/grants/token_transfer_grant.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:sizer/sizer.dart';

part 'token_transfer_grant.g.dart';

class VolumeLimitEntry {
  VolumeLimitEntry({
    required this.id,
    this.amount = '',
    this.windowSeconds = '',
  });

  final int id;
  final String amount;
  final String windowSeconds;

  VolumeLimitEntry copyWith({String? amount, String? windowSeconds}) =>
      VolumeLimitEntry(
        id: id,
        amount: amount ?? this.amount,
        windowSeconds: windowSeconds ?? this.windowSeconds,
      );
}

@riverpod
class TokenGrantLimits extends _$TokenGrantLimits {
  int _nextId = 0;
  int _newId() => _nextId++;

  @override
  List<VolumeLimitEntry> build() => [VolumeLimitEntry(id: _newId())];

  void add() => state = [...state, VolumeLimitEntry(id: _newId())];

  void update(int index, VolumeLimitEntry entry) {
    final updated = [...state];
    updated[index] = entry;
    state = updated;
  }

  void remove(int index) => state = [...state]..removeAt(index);
}

class TokenTransferGrantHandler implements GrantFormHandler {
  const TokenTransferGrantHandler();

  @override
  Widget buildForm(BuildContext context, WidgetRef ref) =>
      const _TokenTransferForm();

  @override
  SpecificGrant buildSpecificGrant(
    Map<String, dynamic> formValues,
    WidgetRef ref,
  ) {
    final limits = ref.read(tokenGrantLimitsProvider);
    final targetText = formValues['tokenTarget'] as String? ?? '';

    return SpecificGrant(
      tokenTransfer: TokenTransferSettings(
        tokenContract: parseHexAddress(
          formValues['tokenContract'] as String? ?? '',
        ),
        target: targetText.trim().isEmpty ? null : parseHexAddress(targetText),
        volumeLimits: limits
            .where(
              (e) =>
                  e.amount.trim().isNotEmpty &&
                  e.windowSeconds.trim().isNotEmpty,
            )
            .map(
              (e) => VolumeRateLimit(
                maxVolume: parseBigIntBytes(e.amount),
                windowSecs: Int64.parseInt(e.windowSeconds),
              ),
            )
            .toList(),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Form widget
// ---------------------------------------------------------------------------

class _TokenTransferForm extends ConsumerWidget {
  const _TokenTransferForm();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final limits = ref.watch(tokenGrantLimitsProvider);
    final notifier = ref.read(tokenGrantLimitsProvider.notifier);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormBuilderTextField(
          name: 'tokenContract',
          decoration: const InputDecoration(
            labelText: 'Token contract',
            hintText: '0x...',
            border: OutlineInputBorder(),
          ),
        ),
        SizedBox(height: 1.6.h),
        FormBuilderTextField(
          name: 'tokenTarget',
          decoration: const InputDecoration(
            labelText: 'Token recipient',
            hintText: '0x... or leave empty for any recipient',
            border: OutlineInputBorder(),
          ),
        ),
        SizedBox(height: 1.6.h),
        _TokenVolumeLimitsField(
          values: limits,
          onAdd: notifier.add,
          onUpdate: notifier.update,
          onRemove: notifier.remove,
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Volume limits list widget
// ---------------------------------------------------------------------------

class _TokenVolumeLimitsField extends StatelessWidget {
  const _TokenVolumeLimitsField({
    required this.values,
    required this.onAdd,
    required this.onUpdate,
    required this.onRemove,
  });

  final List<VolumeLimitEntry> values;
  final VoidCallback onAdd;
  final void Function(int index, VolumeLimitEntry entry) onUpdate;
  final void Function(int index) onRemove;

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
                style: Theme.of(
                  context,
                ).textTheme.labelLarge?.copyWith(fontWeight: FontWeight.w800),
              ),
            ),
            TextButton.icon(
              onPressed: onAdd,
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
              key: ValueKey(values[i].id),
              value: values[i],
              onChanged: (entry) => onUpdate(i, entry),
              onRemove: values.length == 1 ? null : () => onRemove(i),
            ),
          ),
      ],
    );
  }
}

class _TokenVolumeLimitRow extends HookWidget {
  const _TokenVolumeLimitRow({
    super.key,
    required this.value,
    required this.onChanged,
    required this.onRemove,
  });

  final VolumeLimitEntry value;
  final ValueChanged<VolumeLimitEntry> onChanged;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final amountController = useTextEditingController(text: value.amount);
    final windowController = useTextEditingController(
      text: value.windowSeconds,
    );

    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: amountController,
            onChanged: (next) => onChanged(value.copyWith(amount: next)),
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
            onChanged: (next) => onChanged(value.copyWith(windowSeconds: next)),
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
