import 'package:arbiter/features/callouts/callout_event.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'active_callout.freezed.dart';

@freezed
abstract class ActiveCallout with _$ActiveCallout {
  const factory ActiveCallout({
    required String id,
    required String title,
    required String description,
    String? iconUrl,
    required DateTime addedAt,
    required CalloutData data,
  }) = _ActiveCallout;
}
