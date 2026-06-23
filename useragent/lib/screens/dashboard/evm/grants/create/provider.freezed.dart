// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'provider.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$GrantCreationState {

 int? get selectedClientId; SpecificGrant_Grant get grantType;
/// Create a copy of GrantCreationState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GrantCreationStateCopyWith<GrantCreationState> get copyWith => _$GrantCreationStateCopyWithImpl<GrantCreationState>(this as GrantCreationState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GrantCreationState&&(identical(other.selectedClientId, selectedClientId) || other.selectedClientId == selectedClientId)&&(identical(other.grantType, grantType) || other.grantType == grantType));
}


@override
int get hashCode => Object.hash(runtimeType,selectedClientId,grantType);

@override
String toString() {
  return 'GrantCreationState(selectedClientId: $selectedClientId, grantType: $grantType)';
}


}

/// @nodoc
abstract mixin class $GrantCreationStateCopyWith<$Res>  {
  factory $GrantCreationStateCopyWith(GrantCreationState value, $Res Function(GrantCreationState) _then) = _$GrantCreationStateCopyWithImpl;
@useResult
$Res call({
 int? selectedClientId, SpecificGrant_Grant grantType
});




}
/// @nodoc
class _$GrantCreationStateCopyWithImpl<$Res>
    implements $GrantCreationStateCopyWith<$Res> {
  _$GrantCreationStateCopyWithImpl(this._self, this._then);

  final GrantCreationState _self;
  final $Res Function(GrantCreationState) _then;

/// Create a copy of GrantCreationState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? selectedClientId = freezed,Object? grantType = null,}) {
  return _then(_self.copyWith(
selectedClientId: freezed == selectedClientId ? _self.selectedClientId : selectedClientId // ignore: cast_nullable_to_non_nullable
as int?,grantType: null == grantType ? _self.grantType : grantType // ignore: cast_nullable_to_non_nullable
as SpecificGrant_Grant,
  ));
}

}


/// Adds pattern-matching-related methods to [GrantCreationState].
extension GrantCreationStatePatterns on GrantCreationState {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _GrantCreationState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _GrantCreationState() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _GrantCreationState value)  $default,){
final _that = this;
switch (_that) {
case _GrantCreationState():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _GrantCreationState value)?  $default,){
final _that = this;
switch (_that) {
case _GrantCreationState() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int? selectedClientId,  SpecificGrant_Grant grantType)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _GrantCreationState() when $default != null:
return $default(_that.selectedClientId,_that.grantType);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int? selectedClientId,  SpecificGrant_Grant grantType)  $default,) {final _that = this;
switch (_that) {
case _GrantCreationState():
return $default(_that.selectedClientId,_that.grantType);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int? selectedClientId,  SpecificGrant_Grant grantType)?  $default,) {final _that = this;
switch (_that) {
case _GrantCreationState() when $default != null:
return $default(_that.selectedClientId,_that.grantType);case _:
  return null;

}
}

}

/// @nodoc


class _GrantCreationState implements GrantCreationState {
  const _GrantCreationState({this.selectedClientId, this.grantType = SpecificGrant_Grant.etherTransfer});
  

@override final  int? selectedClientId;
@override@JsonKey() final  SpecificGrant_Grant grantType;

/// Create a copy of GrantCreationState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$GrantCreationStateCopyWith<_GrantCreationState> get copyWith => __$GrantCreationStateCopyWithImpl<_GrantCreationState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _GrantCreationState&&(identical(other.selectedClientId, selectedClientId) || other.selectedClientId == selectedClientId)&&(identical(other.grantType, grantType) || other.grantType == grantType));
}


@override
int get hashCode => Object.hash(runtimeType,selectedClientId,grantType);

@override
String toString() {
  return 'GrantCreationState(selectedClientId: $selectedClientId, grantType: $grantType)';
}


}

/// @nodoc
abstract mixin class _$GrantCreationStateCopyWith<$Res> implements $GrantCreationStateCopyWith<$Res> {
  factory _$GrantCreationStateCopyWith(_GrantCreationState value, $Res Function(_GrantCreationState) _then) = __$GrantCreationStateCopyWithImpl;
@override @useResult
$Res call({
 int? selectedClientId, SpecificGrant_Grant grantType
});




}
/// @nodoc
class __$GrantCreationStateCopyWithImpl<$Res>
    implements _$GrantCreationStateCopyWith<$Res> {
  __$GrantCreationStateCopyWithImpl(this._self, this._then);

  final _GrantCreationState _self;
  final $Res Function(_GrantCreationState) _then;

/// Create a copy of GrantCreationState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? selectedClientId = freezed,Object? grantType = null,}) {
  return _then(_GrantCreationState(
selectedClientId: freezed == selectedClientId ? _self.selectedClientId : selectedClientId // ignore: cast_nullable_to_non_nullable
as int?,grantType: null == grantType ? _self.grantType : grantType // ignore: cast_nullable_to_non_nullable
as SpecificGrant_Grant,
  ));
}


}

// dart format on
