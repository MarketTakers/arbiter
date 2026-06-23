# Grant Creation Screen Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the monolithic `create/screen.dart` into a Riverpod-managed provider, composable `flutter_form_builder` field widgets, and per-grant-type handlers with a clean interface.

**Architecture:** A `GrantCreation` provider manages only the state that cannot live in a form field: `selectedClientId` (drives wallet-access filtering) and `grantType` (SegmentedButton). All other shared settings — including validity dates — are `FormBuilder` fields; the custom `FormBuilderDateTimeField` wraps the native date/time picker dialog. Token volume limits are owned by a `TokenGrantLimits` provider that lives entirely inside `token_transfer_grant.dart`. Each grant type implements `GrantFormHandler`, whose `buildSpecificGrant` receives `formValues` and a `WidgetRef` so handlers can read their own providers.

**Tech Stack:** Flutter, Riverpod (riverpod_annotation 4.x), flutter_form_builder 10.x, freezed 3.x, flutter_hooks, Protobuf (Dart), fixnum

---

## File Map

| Path | Action | Responsibility |
|------|--------|----------------|
| `create/utils.dart` | Create | Parsing/conversion helpers (hex addresses, big-int bytes, timestamps, rate limits) |
| `create/provider.dart` | Create | `GrantCreationState` (freezed), `GrantCreation` (@riverpod) — only `selectedClientId` + `grantType` |
| `create/fields/client_picker_field.dart` | Create | `FormBuilderDropdown` for SDK client; syncs selection to `GrantCreation` provider |
| `create/fields/wallet_access_picker_field.dart` | Create | `FormBuilderDropdown` for wallet access; filters by provider's `selectedClientId` |
| `create/fields/chain_id_field.dart` | Create | `FormBuilderTextField` for chain ID |
| `create/fields/date_time_field.dart` | Create | Custom `FormBuilderDateTimeField` — wraps date+time picker dialogs as a `FormBuilderField<DateTime?>` |
| `create/fields/validity_window_field.dart` | Create | Row of two `FormBuilderDateTimeField` widgets (names `validFrom`, `validUntil`) |
| `create/fields/gas_fee_options_field.dart` | Create | Two `FormBuilderTextField` fields for gas fee and priority fee |
| `create/fields/transaction_rate_limit_field.dart` | Create | Two `FormBuilderTextField` fields for tx count and window |
| `create/shared_grant_fields.dart` | Create | Composes all shared field widgets |
| `create/grants/grant_form_handler.dart` | Create | Abstract `GrantFormHandler` with `buildSpecificGrant(formValues, WidgetRef)` |
| `create/grants/ether_transfer_grant.dart` | Create | `EtherTransferGrantHandler` + `_EtherTransferForm` |
| `create/grants/token_transfer_grant.dart` | Create | `TokenGrantLimits` provider, `TokenTransferGrantHandler`, `_TokenTransferForm` + volume limit widgets |
| `create/screen.dart` | Modify | Thin orchestration: `FormBuilder` root, section layout, submit logic, dispatch handler |

---

## Task 1: Create `utils.dart` — Parsing helpers

**Files:**
- Create: `lib/screens/dashboard/evm/grants/create/utils.dart`

- [ ] **Step 1: Create `utils.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/utils.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:fixnum/fixnum.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

Timestamp toTimestamp(DateTime value) {
  final utc = value.toUtc();
  return Timestamp()
    ..seconds = Int64(utc.millisecondsSinceEpoch ~/ 1000)
    ..nanos = (utc.microsecondsSinceEpoch % 1000000) * 1000;
}

TransactionRateLimit? buildRateLimit(String countText, String windowText) {
  if (countText.trim().isEmpty || windowText.trim().isEmpty) return null;
  return TransactionRateLimit(
    count: int.parse(countText.trim()),
    windowSecs: Int64.parseInt(windowText.trim()),
  );
}

VolumeRateLimit? buildVolumeLimit(String amountText, String windowText) {
  if (amountText.trim().isEmpty || windowText.trim().isEmpty) return null;
  return VolumeRateLimit(
    maxVolume: parseBigIntBytes(amountText),
    windowSecs: Int64.parseInt(windowText.trim()),
  );
}

List<int>? optionalBigIntBytes(String value) {
  if (value.trim().isEmpty) return null;
  return parseBigIntBytes(value);
}

List<int> parseBigIntBytes(String value) {
  final number = BigInt.parse(value.trim());
  if (number < BigInt.zero) throw Exception('Numeric values must be positive.');
  if (number == BigInt.zero) return [0];
  var remaining = number;
  final bytes = <int>[];
  while (remaining > BigInt.zero) {
    bytes.insert(0, (remaining & BigInt.from(0xff)).toInt());
    remaining >>= 8;
  }
  return bytes;
}

List<List<int>> parseAddresses(String input) {
  final parts = input
      .split(RegExp(r'[\n,]'))
      .map((p) => p.trim())
      .where((p) => p.isNotEmpty);
  return parts.map(parseHexAddress).toList();
}

List<int> parseHexAddress(String value) {
  final normalized = value.trim().replaceFirst(RegExp(r'^0x'), '');
  if (normalized.length != 40) throw Exception('Expected a 20-byte hex address.');
  return [
    for (var i = 0; i < normalized.length; i += 2)
      int.parse(normalized.substring(i, i + 2), radix: 16),
  ];
}

String shortAddress(List<int> bytes) {
  final hex = bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  return '0x${hex.substring(0, 6)}...${hex.substring(hex.length - 4)}';
}
```

- [ ] **Step 2: Verify**

```sh
cd operator && dart analyze lib/screens/dashboard/evm/grants/create/utils.dart
```

Expected: no errors.

- [ ] **Step 3: Commit**

```sh
jj describe -m "refactor(grants): extract parsing helpers to utils.dart"
```

---

## Task 2: Create `provider.dart` — Grant creation state

**Files:**
- Create: `lib/screens/dashboard/evm/grants/create/provider.dart`
- Create (generated): `lib/screens/dashboard/evm/grants/create/provider.freezed.dart`
- Create (generated): `lib/screens/dashboard/evm/grants/create/provider.g.dart`

The provider holds only what cannot live in a `FormBuilder` field:
- `selectedClientId` — a helper for filtering the wallet access dropdown; not part of `SharedSettings` proto
- `grantType` — driven by a `SegmentedButton`, not a form input

- [ ] **Step 1: Create `provider.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/provider.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'provider.freezed.dart';
part 'provider.g.dart';

@freezed
abstract class GrantCreationState with _$GrantCreationState {
  const factory GrantCreationState({
    int? selectedClientId,
    @Default(SpecificGrant_Grant.etherTransfer) SpecificGrant_Grant grantType,
  }) = _GrantCreationState;
}

@riverpod
class GrantCreation extends _$GrantCreation {
  @override
  GrantCreationState build() => const GrantCreationState();

  void setClientId(int? id) => state = state.copyWith(selectedClientId: id);
  void setGrantType(SpecificGrant_Grant type) =>
      state = state.copyWith(grantType: type);
}
```

- [ ] **Step 2: Run code generator**

```sh
cd operator && dart run build_runner build --delete-conflicting-outputs
```

Expected: generates `provider.freezed.dart` and `provider.g.dart`, no errors.

- [ ] **Step 3: Verify**

```sh
cd operator && dart analyze lib/screens/dashboard/evm/grants/create/provider.dart
```

Expected: no errors.

- [ ] **Step 4: Commit**

```sh
jj describe -m "feat(grants): add GrantCreation provider (client selection + grant type)"
```

---

## Task 3: Create field widgets

**Files:**
- Create: `lib/screens/dashboard/evm/grants/create/fields/client_picker_field.dart`
- Create: `lib/screens/dashboard/evm/grants/create/fields/wallet_access_picker_field.dart`
- Create: `lib/screens/dashboard/evm/grants/create/fields/chain_id_field.dart`
- Create: `lib/screens/dashboard/evm/grants/create/fields/date_time_field.dart`
- Create: `lib/screens/dashboard/evm/grants/create/fields/validity_window_field.dart`
- Create: `lib/screens/dashboard/evm/grants/create/fields/gas_fee_options_field.dart`
- Create: `lib/screens/dashboard/evm/grants/create/fields/transaction_rate_limit_field.dart`

- [ ] **Step 1: Create `client_picker_field.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/fields/client_picker_field.dart
import 'package:arbiter/proto/operator.pb.dart';
import 'package:arbiter/providers/sdk_clients/list.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/provider.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class ClientPickerField extends ConsumerWidget {
  const ClientPickerField({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final clients =
        ref.watch(sdkClientsProvider).asData?.value ?? const <SdkClientEntry>[];

    return FormBuilderDropdown<int>(
      name: 'clientId',
      decoration: const InputDecoration(
        labelText: 'Client',
        border: OutlineInputBorder(),
      ),
      items: [
        for (final c in clients)
          DropdownMenuItem(
            value: c.id,
            child: Text(c.info.name.isEmpty ? 'Client #${c.id}' : c.info.name),
          ),
      ],
      onChanged: clients.isEmpty
          ? null
          : (value) =>
              ref.read(grantCreationProvider.notifier).setClientId(value),
    );
  }
}
```

- [ ] **Step 2: Create `wallet_access_picker_field.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/fields/wallet_access_picker_field.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/operator.pb.dart';
import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access_list.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/provider.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class WalletAccessPickerField extends ConsumerWidget {
  const WalletAccessPickerField({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(grantCreationProvider);
    final allAccesses =
        ref.watch(walletAccessListProvider).asData?.value ??
        const <SdkClientWalletAccess>[];
    final wallets =
        ref.watch(evmProvider).asData?.value ?? const <WalletEntry>[];

    final walletById = <int, WalletEntry>{for (final w in wallets) w.id: w};
    final accesses = state.selectedClientId == null
        ? const <SdkClientWalletAccess>[]
        : allAccesses
              .where((a) => a.access.sdkClientId == state.selectedClientId)
              .toList();

    return FormBuilderDropdown<int>(
      name: 'walletAccessId',
      decoration: InputDecoration(
        labelText: 'Wallet access',
        helperText: state.selectedClientId == null
            ? 'Select a client first'
            : accesses.isEmpty
                ? 'No wallet accesses for this client'
                : null,
        border: const OutlineInputBorder(),
      ),
      items: [
        for (final a in accesses)
          DropdownMenuItem(
            value: a.id,
            child: Text(() {
              final wallet = walletById[a.access.walletId];
              return wallet != null
                  ? shortAddress(wallet.address)
                  : 'Wallet #${a.access.walletId}';
            }()),
          ),
      ],
    );
  }
}
```

- [ ] **Step 3: Create `chain_id_field.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/fields/chain_id_field.dart
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';

class ChainIdField extends StatelessWidget {
  const ChainIdField({super.key});

  @override
  Widget build(BuildContext context) {
    return FormBuilderTextField(
      name: 'chainId',
      initialValue: '1',
      keyboardType: TextInputType.number,
      decoration: const InputDecoration(
        labelText: 'Chain ID',
        hintText: '1',
        border: OutlineInputBorder(),
      ),
    );
  }
}
```

- [ ] **Step 4: Create `date_time_field.dart`**

`flutter_form_builder` has no built-in date+time picker that matches the existing UX (separate date then time dialog, long-press to clear). Implement one via `FormBuilderField<DateTime?>`.

```dart
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
                final now = DateTime.now();
                final date = await showDatePicker(
                  context: field.context,
                  firstDate: DateTime(now.year - 5),
                  lastDate: DateTime(now.year + 10),
                  initialDate: value ?? now,
                );
                if (date == null) return;
                // ignore: use_build_context_synchronously — field.context is
                // still valid as long as the widget is in the tree.
                if (!field.context.mounted) return;
                final time = await showTimePicker(
                  context: field.context,
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
```

- [ ] **Step 5: Create `validity_window_field.dart`**

```dart
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
```

- [ ] **Step 6: Create `gas_fee_options_field.dart`**

```dart
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
```

- [ ] **Step 7: Create `transaction_rate_limit_field.dart`**

```dart
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
```

- [ ] **Step 8: Verify all field widgets**

```sh
cd operator && dart analyze lib/screens/dashboard/evm/grants/create/fields/
```

Expected: no errors.

- [ ] **Step 9: Commit**

```sh
jj describe -m "feat(grants): add composable FormBuilder field widgets incl. custom DateTimeField"
```

---

## Task 4: Create `SharedGrantFields` widget

**Files:**
- Create: `lib/screens/dashboard/evm/grants/create/shared_grant_fields.dart`

- [ ] **Step 1: Create `shared_grant_fields.dart`**

```dart
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
```

- [ ] **Step 2: Verify**

```sh
cd operator && dart analyze lib/screens/dashboard/evm/grants/create/shared_grant_fields.dart
```

Expected: no errors.

- [ ] **Step 3: Commit**

```sh
jj describe -m "feat(grants): add SharedGrantFields composite widget"
```

---

## Task 5: Create grant form handlers

**Files:**
- Create: `lib/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart`
- Create: `lib/screens/dashboard/evm/grants/create/grants/ether_transfer_grant.dart`
- Create: `lib/screens/dashboard/evm/grants/create/grants/token_transfer_grant.dart`
- Create (generated): `lib/screens/dashboard/evm/grants/create/grants/token_transfer_grant.g.dart`

- [ ] **Step 1: Create `grant_form_handler.dart`**

`buildSpecificGrant` takes `WidgetRef` so each handler can read its own providers (e.g., token volume limits) without coupling to a shared state object.

```dart
// lib/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:flutter/widgets.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

abstract class GrantFormHandler {
  /// Renders the grant-specific form section.
  ///
  /// The returned widget must be a descendant of the [FormBuilder] in the
  /// screen so its [FormBuilderField] children register automatically.
  Widget buildForm(BuildContext context, WidgetRef ref);

  /// Assembles a [SpecificGrant] proto.
  ///
  /// [formValues] — `formKey.currentState!.value` after `saveAndValidate()`.
  /// [ref] — read any provider the handler owns (e.g. token volume limits).
  SpecificGrant buildSpecificGrant(
    Map<String, dynamic> formValues,
    WidgetRef ref,
  );
}
```

- [ ] **Step 2: Create `ether_transfer_grant.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/grants/ether_transfer_grant.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

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
    return SpecificGrant(
      etherTransfer: EtherTransferSettings(
        targets: parseAddresses(formValues['etherRecipients'] as String? ?? ''),
        limit: buildVolumeLimit(
          formValues['etherVolume'] as String? ?? '',
          formValues['etherVolumeWindow'] as String? ?? '',
        ),
      ),
    );
  }
}

class _EtherTransferForm extends StatelessWidget {
  const _EtherTransferForm();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FormBuilderTextField(
          name: 'etherRecipients',
          minLines: 3,
          maxLines: 6,
          decoration: const InputDecoration(
            labelText: 'Ether recipients',
            hintText:
                'One 0x address per line. Leave empty for unrestricted targets.',
            border: OutlineInputBorder(),
          ),
        ),
        SizedBox(height: 1.6.h),
        Text(
          'Ether volume limit',
          style: Theme.of(context).textTheme.labelLarge?.copyWith(
                fontWeight: FontWeight.w800,
              ),
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
```

- [ ] **Step 3: Create `token_transfer_grant.dart`**

`TokenGrantLimits` is a scoped `@riverpod` provider (auto-dispose) that owns the dynamic volume limit list for the token grant form. `TokenTransferGrantHandler.buildSpecificGrant` reads it via `ref`.

```dart
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

// ---------------------------------------------------------------------------
// Volume limit entry — a single row's data
// ---------------------------------------------------------------------------

class VolumeLimitEntry {
  const VolumeLimitEntry({this.amount = '', this.windowSeconds = ''});

  final String amount;
  final String windowSeconds;

  VolumeLimitEntry copyWith({String? amount, String? windowSeconds}) =>
      VolumeLimitEntry(
        amount: amount ?? this.amount,
        windowSeconds: windowSeconds ?? this.windowSeconds,
      );
}

// ---------------------------------------------------------------------------
// Provider — owns token volume limits; auto-disposed when screen pops
// ---------------------------------------------------------------------------

@riverpod
class TokenGrantLimits extends _$TokenGrantLimits {
  @override
  List<VolumeLimitEntry> build() => const [VolumeLimitEntry()];

  void add() => state = [...state, const VolumeLimitEntry()];

  void update(int index, VolumeLimitEntry entry) {
    final updated = [...state];
    updated[index] = entry;
    state = updated;
  }

  void remove(int index) => state = [...state]..removeAt(index);
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

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
        tokenContract:
            parseHexAddress(formValues['tokenContract'] as String? ?? ''),
        target: targetText.trim().isEmpty ? null : parseHexAddress(targetText),
        volumeLimits: limits
            .where((e) => e.amount.trim().isNotEmpty)
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
                style: Theme.of(context).textTheme.labelLarge?.copyWith(
                      fontWeight: FontWeight.w800,
                    ),
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
              key: ValueKey(i),
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
    final windowController = useTextEditingController(text: value.windowSeconds);

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
```

- [ ] **Step 4: Run code generator for token_transfer_grant.g.dart**

```sh
cd operator && dart run build_runner build --delete-conflicting-outputs
```

Expected: generates `token_transfer_grant.g.dart`, no errors.

- [ ] **Step 5: Verify**

```sh
cd operator && dart analyze lib/screens/dashboard/evm/grants/create/grants/
```

Expected: no errors.

- [ ] **Step 6: Commit**

```sh
jj describe -m "feat(grants): add GrantFormHandler interface and per-type implementations"
```

---

## Task 6: Rewrite `screen.dart`

**Files:**
- Modify: `lib/screens/dashboard/evm/grants/create/screen.dart`

The screen owns `FormBuilder`, dispatches to the active handler, and assembles `SharedSettings` on submit. `validFrom`/`validUntil` are now read from `formValues['validFrom']` etc. — no more provider reads for dates.

- [ ] **Step 1: Replace `screen.dart`**

```dart
// lib/screens/dashboard/evm/grants/create/screen.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/providers/evm/evm_grants.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/ether_transfer_grant.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/grants/token_transfer_grant.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/provider.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/shared_grant_fields.dart';
import 'package:arbiter/screens/dashboard/evm/grants/create/utils.dart';
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
        if (validFrom != null) sharedSettings.validFrom = toTimestamp(validFrom);
        if (validUntil != null) {
          sharedSettings.validUntil = toTimestamp(validUntil);
        }
        final gasBytes =
            optionalBigIntBytes(formValues['maxGasFeePerGas'] as String? ?? '');
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
                title: 'Shared grant options',
                child: SharedGrantFields(),
              ),
              SizedBox(height: 1.8.h),
              _GrantTypeSelector(
                value: state.grantType,
                onChanged: notifier.setGrantType,
              ),
              SizedBox(height: 1.8.h),
              _Section(
                title: 'Grant-specific options',
                child: handler.buildForm(context, ref),
              ),
              SizedBox(height: 2.2.h),
              Align(
                alignment: Alignment.centerRight,
                child: FilledButton.icon(
                  onPressed:
                      createMutation is MutationPending ? null : submit,
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
          colors: [Color(0xFFF7F9FC), Color(0xFFFDF5EA)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        border: Border.all(color: const Color(0x1A17324A)),
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
  const _Section({required this.title, required this.child});

  final String title;
  final Widget child;

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
```

- [ ] **Step 2: Verify the full create/ directory**

```sh
cd operator && dart analyze lib/screens/dashboard/evm/grants/create/
```

Expected: no errors.

- [ ] **Step 3: Commit**

```sh
jj describe -m "refactor(grants): decompose CreateEvmGrantScreen into provider + field widgets + grant handlers"
```

---

## Self-Review

### Spec coverage

| Requirement | Task |
|-------------|------|
| Riverpod provider for grant creation state | Task 2 (`GrantCreation`) |
| flutter_form_builder in composable manner | Tasks 3–4 |
| `fields/` folder for shared fields | Task 3 |
| Custom `FormBuilderDateTimeField` (no built-in) | Task 3, Step 4 |
| `validFrom`/`validUntil` as form fields | Task 3 (`ValidityWindowField` uses `FormBuilderDateTimeField`) |
| `tokenVolumeLimits` in grant-specific implementation | Task 5 (`TokenGrantLimits` in `token_transfer_grant.dart`) |
| Grant handler with `buildForm` + `buildSpecificGrant` | Task 5 |
| `SharedGrantFields` widget | Task 4 |
| Thin main screen | Task 6 |
| Class renamed to `GrantCreation` (provider: `grantCreationProvider`) | Task 2 |

### Placeholder scan

No TODOs, TBDs, or "similar to task N" references found.

### Type consistency

- `VolumeLimitEntry` defined in `token_transfer_grant.dart`, used only within that file ✓
- `GrantFormHandler` interface: `buildSpecificGrant(Map<String, dynamic>, WidgetRef)` matches both implementations ✓
- `grantCreationProvider` (from `@riverpod class GrantCreation`) used in `client_picker_field.dart`, `wallet_access_picker_field.dart`, `screen.dart` ✓
- `tokenGrantLimitsProvider` (from `@riverpod class TokenGrantLimits`) used in `_TokenTransferForm.build` and `TokenTransferGrantHandler.buildSpecificGrant` ✓
- FormBuilder field names in `screen.dart` (`validFrom`, `validUntil`, `walletAccessId`, `chainId`, `maxGasFeePerGas`, `maxPriorityFeePerGas`, `txCount`, `txWindow`) match the `name:` params in field widgets ✓
