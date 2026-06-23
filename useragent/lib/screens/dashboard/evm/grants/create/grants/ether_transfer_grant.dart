// lib/screens/dashboard/evm/grants/create/grants/ether_transfer_grant.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:sizer/sizer.dart';

part 'ether_transfer_grant.g.dart';

class EtherTargetEntry {
  EtherTargetEntry({required this.id, this.address = ''});

  final int id;
  final String address;

  EtherTargetEntry copyWith({String? address}) =>
      EtherTargetEntry(id: id, address: address ?? this.address);
}

@riverpod
class EtherGrantTargets extends _$EtherGrantTargets {
  int _nextId = 0;
  int _newId() => _nextId++;

  @override
  List<EtherTargetEntry> build() => [EtherTargetEntry(id: _newId())];

  void add() => state = [...state, EtherTargetEntry(id: _newId())];

  void update(int index, EtherTargetEntry entry) {
    final updated = [...state];
    updated[index] = entry;
    state = updated;
  }

  void remove(int index) => state = [...state]..removeAt(index);
}

class EtherTransferGrantHandler implements GrantFormHandler {
  const EtherTransferGrantHandler();

  @override
  Widget buildForm(BuildContext context, WidgetRef ref) =>
      const _EtherTransferForm();

  @override
  SpecificGrant buildSpecificGrant(
    Map<String, dynamic> formValues,
    WidgetRef ref,
  ) {
    final targets = ref.read(etherGrantTargetsProvider);

    return SpecificGrant(
      etherTransfer: EtherTransferSettings(
        targets: targets
            .where((e) => e.address.trim().isNotEmpty)
            .map((e) => parseHexAddress(e.address))
            .toList(),
        limit: buildVolumeLimit(
          formValues['etherVolume'] as String? ?? '',
          formValues['etherVolumeWindow'] as String? ?? '',
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Form widget
// ---------------------------------------------------------------------------

class _EtherTransferForm extends ConsumerWidget {
  const _EtherTransferForm();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final targets = ref.watch(etherGrantTargetsProvider);
    final notifier = ref.read(etherGrantTargetsProvider.notifier);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _EtherTargetsField(
          values: targets,
          onAdd: notifier.add,
          onUpdate: notifier.update,
          onRemove: notifier.remove,
        ),
        SizedBox(height: 1.6.h),
        Text(
          'Ether volume limit',
          style: Theme.of(
            context,
          ).textTheme.labelLarge?.copyWith(fontWeight: FontWeight.w800),
        ),
        SizedBox(height: 0.8.h),
        Row(
          children: [
            Expanded(
              child: FormBuilderTextField(
                name: 'etherVolume',
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Max volume',
                  hintText: '1000000000000000000',
                  border: OutlineInputBorder(),
                ),
              ),
            ),
            SizedBox(width: 1.w),
            Expanded(
              child: FormBuilderTextField(
                name: 'etherVolumeWindow',
                keyboardType: TextInputType.number,
                decoration: const InputDecoration(
                  labelText: 'Window (seconds)',
                  hintText: '86400',
                  border: OutlineInputBorder(),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Targets list widget
// ---------------------------------------------------------------------------

class _EtherTargetsField extends StatelessWidget {
  const _EtherTargetsField({
    required this.values,
    required this.onAdd,
    required this.onUpdate,
    required this.onRemove,
  });

  final List<EtherTargetEntry> values;
  final VoidCallback onAdd;
  final void Function(int index, EtherTargetEntry entry) onUpdate;
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
                'Ether targets',
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
            child: _EtherTargetRow(
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

class _EtherTargetRow extends HookWidget {
  const _EtherTargetRow({
    super.key,
    required this.value,
    required this.onChanged,
    required this.onRemove,
  });

  final EtherTargetEntry value;
  final ValueChanged<EtherTargetEntry> onChanged;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final addressController = useTextEditingController(text: value.address);

    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: addressController,
            onChanged: (next) => onChanged(value.copyWith(address: next)),
            decoration: const InputDecoration(
              labelText: 'Address',
              hintText: '0x...',
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
