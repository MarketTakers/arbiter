import 'package:arbiter/features/connection/vault.dart';
import 'package:arbiter/proto/shared/vault.pbenum.dart';
import 'package:arbiter/proto/user_agent/vault/bootstrap.pbenum.dart';
import 'package:arbiter/proto/user_agent/vault/unseal.pbenum.dart';
import 'package:arbiter/providers/connection/connection_manager.dart';
import 'package:arbiter/providers/vault_state.dart';
import 'package:arbiter/router.gr.dart';
import 'package:arbiter/widgets/bottom_popup.dart';
import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:sizer/sizer.dart';

@RoutePage()
class VaultSetupScreen extends HookConsumerWidget {
  const VaultSetupScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final vaultState = ref.watch(vaultStateProvider);
    final bootstrapPasswordController = useTextEditingController();
    final bootstrapConfirmController = useTextEditingController();
    final unsealPasswordController = useTextEditingController();
    final errorText = useState<String?>(null);
    final isSubmitting = useState(false);

    useEffect(() {
      if (vaultState.asData?.value == VaultState.VAULT_STATE_UNSEALED) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (context.mounted) {
            context.router.replace(const DashboardRouter());
          }
        });
      }

      return null;
    }, [context, vaultState.asData?.value]);

    Future<void> refreshVaultState() async {
      ref.invalidate(vaultStateProvider);
      await ref.read(vaultStateProvider.future);
    }

    Future<void> submitBootstrap() async {
      final password = bootstrapPasswordController.text;
      final confirmation = bootstrapConfirmController.text;

      if (password.isEmpty || confirmation.isEmpty) {
        errorText.value = 'Enter the password twice.';
        return;
      }

      if (password != confirmation) {
        errorText.value = 'Passwords do not match.';
        return;
      }

      final confirmed = await showBottomPopup<bool>(
        context: context,
        builder: (popupContext) {
          return _WarningPopup(
            title: 'Bootstrap vault?',
            body:
                'This password cannot be recovered. If you lose it, the vault cannot be unsealed.',
            confirmLabel: 'Bootstrap',
            onCancel: () => Navigator.of(popupContext).pop(false),
            onConfirm: () => Navigator.of(popupContext).pop(true),
          );
        },
      );

      if (confirmed != true) {
        return;
      }

      errorText.value = null;
      isSubmitting.value = true;

      try {
        final connection = await ref.read(connectionManagerProvider.future);
        if (connection == null) {
          throw Exception('Not connected to the server.');
        }

        final result = await bootstrapVault(connection, password);
        switch (result) {
          case BootstrapResult.BOOTSTRAP_RESULT_SUCCESS:
            bootstrapPasswordController.clear();
            bootstrapConfirmController.clear();
            await refreshVaultState();
            break;
          case BootstrapResult.BOOTSTRAP_RESULT_ALREADY_BOOTSTRAPPED:
            errorText.value =
                'The vault was already bootstrapped. Refreshing vault state.';
            await refreshVaultState();
            break;
          case BootstrapResult.BOOTSTRAP_RESULT_INVALID_KEY:
          case BootstrapResult.BOOTSTRAP_RESULT_UNSPECIFIED:
            errorText.value = 'Failed to bootstrap the vault.';
            break;
        }
      } catch (error) {
        errorText.value = _formatVaultError(error);
      } finally {
        isSubmitting.value = false;
      }
    }

    Future<void> submitUnseal() async {
      final password = unsealPasswordController.text;
      if (password.isEmpty) {
        errorText.value = 'Enter the vault password.';
        return;
      }

      errorText.value = null;
      isSubmitting.value = true;

      try {
        final connection = await ref.read(connectionManagerProvider.future);
        if (connection == null) {
          throw Exception('Not connected to the server.');
        }

        final result = await unsealVault(connection, password);
        switch (result) {
          case UnsealResult.UNSEAL_RESULT_SUCCESS:
            unsealPasswordController.clear();
            await refreshVaultState();
            break;
          case UnsealResult.UNSEAL_RESULT_INVALID_KEY:
            errorText.value = 'Incorrect password.';
            break;
          case UnsealResult.UNSEAL_RESULT_UNBOOTSTRAPPED:
            errorText.value =
                'The vault is not bootstrapped yet. Refreshing vault state.';
            await refreshVaultState();
            break;
          case UnsealResult.UNSEAL_RESULT_UNSPECIFIED:
            errorText.value = 'Failed to unseal the vault.';
            break;
        }
      } catch (error) {
        errorText.value = _formatVaultError(error);
      } finally {
        isSubmitting.value = false;
      }
    }

    final body = switch (vaultState) {
      AsyncLoading() => const Center(child: CircularProgressIndicator()),
      AsyncError(:final error) => _VaultCard(
        title: 'Vault unavailable',
        subtitle: _formatVaultError(error),
        child: FilledButton(
          onPressed: () => ref.invalidate(vaultStateProvider),
          child: const Text('Retry'),
        ),
      ),
      AsyncData(:final value) => switch (value) {
        VaultState.VAULT_STATE_UNBOOTSTRAPPED => _VaultCard(
          title: 'Create vault password',
          subtitle:
              'Choose the password that will be required to unseal this vault.',
          child: _PasswordForm(
            errorText: errorText.value,
            isSubmitting: isSubmitting.value,
            submitLabel: 'Bootstrap vault',
            onSubmit: submitBootstrap,
            fields: [
              _PasswordFieldConfig(
                controller: bootstrapPasswordController,
                label: 'Password',
                textInputAction: TextInputAction.next,
              ),
              _PasswordFieldConfig(
                controller: bootstrapConfirmController,
                label: 'Confirm password',
                textInputAction: TextInputAction.done,
                onSubmitted: (_) => submitBootstrap(),
              ),
            ],
          ),
        ),
        VaultState.VAULT_STATE_SEALED => _VaultCard(
          title: 'Unseal vault',
          subtitle: 'Enter the vault password to continue.',
          child: _PasswordForm(
            errorText: errorText.value,
            isSubmitting: isSubmitting.value,
            submitLabel: 'Unseal vault',
            onSubmit: submitUnseal,
            fields: [
              _PasswordFieldConfig(
                controller: unsealPasswordController,
                label: 'Password',
                textInputAction: TextInputAction.done,
                onSubmitted: (_) => submitUnseal(),
              ),
            ],
          ),
        ),
        VaultState.VAULT_STATE_UNSEALED => const Center(
          child: CircularProgressIndicator(),
        ),
        VaultState.VAULT_STATE_ERROR => _VaultCard(
          title: 'Vault state unavailable',
          subtitle: 'Unable to determine the current vault state.',
          child: FilledButton(
            onPressed: () => ref.invalidate(vaultStateProvider),
            child: const Text('Retry'),
          ),
        ),
        VaultState.VAULT_STATE_UNSPECIFIED => _VaultCard(
          title: 'Vault state unavailable',
          subtitle: 'Unable to determine the current vault state.',
          child: FilledButton(
            onPressed: () => ref.invalidate(vaultStateProvider),
            child: const Text('Retry'),
          ),
        ),
        null => _VaultCard(
          title: 'Vault state unavailable',
          subtitle: 'Unable to determine the current vault state.',
          child: FilledButton(
            onPressed: () => ref.invalidate(vaultStateProvider),
            child: const Text('Retry'),
          ),
        ),
        _ => _VaultCard(
          title: 'Vault state unavailable',
          subtitle: 'Unable to determine the current vault state.',
          child: FilledButton(
            onPressed: () => ref.invalidate(vaultStateProvider),
            child: const Text('Retry'),
          ),
        ),
      },
    };

    return Scaffold(
      appBar: AppBar(title: const Text('Vault Setup')),
      body: Center(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 6.w, vertical: 3.h),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 520),
            child: body,
          ),
        ),
      ),
    );
  }
}

class _VaultCard extends StatelessWidget {
  const _VaultCard({
    required this.title,
    required this.subtitle,
    required this.child,
  });

  final String title;
  final String subtitle;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 12),
            Text(subtitle, style: Theme.of(context).textTheme.bodyMedium),
            const SizedBox(height: 24),
            child,
          ],
        ),
      ),
    );
  }
}

class _PasswordForm extends StatelessWidget {
  const _PasswordForm({
    required this.fields,
    required this.errorText,
    required this.isSubmitting,
    required this.submitLabel,
    required this.onSubmit,
  });

  final List<_PasswordFieldConfig> fields;
  final String? errorText;
  final bool isSubmitting;
  final String submitLabel;
  final Future<void> Function() onSubmit;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final field in fields) ...[
          TextField(
            controller: field.controller,
            obscureText: true,
            enableSuggestions: false,
            autocorrect: false,
            textInputAction: field.textInputAction,
            onSubmitted: field.onSubmitted,
            decoration: InputDecoration(
              border: const OutlineInputBorder(),
              labelText: field.label,
            ),
          ),
          const SizedBox(height: 16),
        ],
        if (errorText != null) ...[
          Text(
            errorText!,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
          const SizedBox(height: 16),
        ],
        Align(
          alignment: Alignment.centerLeft,
          child: FilledButton(
            onPressed: isSubmitting ? null : () => onSubmit(),
            child: Text(isSubmitting ? 'Working...' : submitLabel),
          ),
        ),
      ],
    );
  }
}

class _PasswordFieldConfig {
  const _PasswordFieldConfig({
    required this.controller,
    required this.label,
    required this.textInputAction,
    this.onSubmitted,
  });

  final TextEditingController controller;
  final String label;
  final TextInputAction textInputAction;
  final ValueChanged<String>? onSubmitted;
}

class _WarningPopup extends StatelessWidget {
  const _WarningPopup({
    required this.title,
    required this.body,
    required this.confirmLabel,
    required this.onCancel,
    required this.onConfirm,
  });

  final String title;
  final String body;
  final String confirmLabel;
  final VoidCallback onCancel;
  final VoidCallback onConfirm;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Theme.of(context).colorScheme.surface,
      borderRadius: BorderRadius.circular(24),
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 12),
            Text(body, style: Theme.of(context).textTheme.bodyMedium),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(onPressed: onCancel, child: const Text('Cancel')),
                const SizedBox(width: 12),
                FilledButton(onPressed: onConfirm, child: Text(confirmLabel)),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

String _formatVaultError(Object error) {
  final message = error.toString();

  if (message.contains('GrpcError')) {
    return 'The server rejected the vault request. Check the password and try again.';
  }

  return message.replaceFirst('Exception: ', '');
}
