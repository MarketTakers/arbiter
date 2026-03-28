// lib/screens/dashboard/evm/grants/create/grants/grant_form_handler.dart
import 'package:arbiter/proto/evm.pb.dart';
import 'package:flutter/widgets.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

abstract class GrantFormHandler {
  /// Renders the grant-specific form section.
  ///
  /// The returned widget must be a descendant of the [FormBuilder] in the
  /// screen so its [FormBuilderField] children register automatically.
  ///
  /// **Field name contract:** All `name:` values used by this handler must be
  /// unique across ALL [GrantFormHandler] implementations. [FormBuilder]
  /// retains field state across handler switches, so name collisions cause
  /// silent data corruption.
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
