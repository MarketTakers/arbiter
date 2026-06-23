// lib/screens/dashboard/evm/grants/create/fields/client_picker_field.dart
import 'package:arbiter/proto/user_agent/sdk_client.pb.dart' as ua_sdk;
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
        ref.watch(sdkClientsProvider).asData?.value ?? const <ua_sdk.Entry>[];

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
          : (value) {
              ref.read(grantCreationProvider.notifier).setClientId(value);
              FormBuilder.of(
                context,
              )?.fields['walletAccessId']?.didChange(null);
            },
    );
  }
}
