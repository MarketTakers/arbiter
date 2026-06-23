// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'evm_grants.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$EvmGrantsState {

 List<GrantEntry> get grants; bool get showRevoked;
/// Create a copy of EvmGrantsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EvmGrantsStateCopyWith<EvmGrantsState> get copyWith => _$EvmGrantsStateCopyWithImpl<EvmGrantsState>(this as EvmGrantsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EvmGrantsState&&const DeepCollectionEquality().equals(other.grants, grants)&&(identical(other.showRevoked, showRevoked) || other.showRevoked == showRevoked));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(grants),showRevoked);

@override
String toString() {
  return 'EvmGrantsState(grants: $grants, showRevoked: $showRevoked)';
}


}

/// @nodoc
abstract mixin class $EvmGrantsStateCopyWith<$Res>  {
  factory $EvmGrantsStateCopyWith(EvmGrantsState value, $Res Function(EvmGrantsState) _then) = _$EvmGrantsStateCopyWithImpl;
@useResult
$Res call({
 List<GrantEntry> grants, bool showRevoked
});




}
/// @nodoc
class _$EvmGrantsStateCopyWithImpl<$Res>
    implements $EvmGrantsStateCopyWith<$Res> {
  _$EvmGrantsStateCopyWithImpl(this._self, this._then);

  final EvmGrantsState _self;
  final $Res Function(EvmGrantsState) _then;

/// Create a copy of EvmGrantsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? grants = null,Object? showRevoked = null,}) {
  return _then(_self.copyWith(
grants: null == grants ? _self.grants : grants // ignore: cast_nullable_to_non_nullable
as List<GrantEntry>,showRevoked: null == showRevoked ? _self.showRevoked : showRevoked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [EvmGrantsState].
extension EvmGrantsStatePatterns on EvmGrantsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _EvmGrantsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _EvmGrantsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _EvmGrantsState value)  $default,){
final _that = this;
switch (_that) {
case _EvmGrantsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _EvmGrantsState value)?  $default,){
final _that = this;
switch (_that) {
case _EvmGrantsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<GrantEntry> grants,  bool showRevoked)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _EvmGrantsState() when $default != null:
return $default(_that.grants,_that.showRevoked);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<GrantEntry> grants,  bool showRevoked)  $default,) {final _that = this;
switch (_that) {
case _EvmGrantsState():
return $default(_that.grants,_that.showRevoked);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<GrantEntry> grants,  bool showRevoked)?  $default,) {final _that = this;
switch (_that) {
case _EvmGrantsState() when $default != null:
return $default(_that.grants,_that.showRevoked);case _:
  return null;

}
}

}

/// @nodoc


class _EvmGrantsState extends EvmGrantsState {
  const _EvmGrantsState({required final  List<GrantEntry> grants, this.showRevoked = false}): _grants = grants,super._();
  

 final  List<GrantEntry> _grants;
@override List<GrantEntry> get grants {
  if (_grants is EqualUnmodifiableListView) return _grants;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_grants);
}

@override@JsonKey() final  bool showRevoked;

/// Create a copy of EvmGrantsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$EvmGrantsStateCopyWith<_EvmGrantsState> get copyWith => __$EvmGrantsStateCopyWithImpl<_EvmGrantsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _EvmGrantsState&&const DeepCollectionEquality().equals(other._grants, _grants)&&(identical(other.showRevoked, showRevoked) || other.showRevoked == showRevoked));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_grants),showRevoked);

@override
String toString() {
  return 'EvmGrantsState(grants: $grants, showRevoked: $showRevoked)';
}


}

/// @nodoc
abstract mixin class _$EvmGrantsStateCopyWith<$Res> implements $EvmGrantsStateCopyWith<$Res> {
  factory _$EvmGrantsStateCopyWith(_EvmGrantsState value, $Res Function(_EvmGrantsState) _then) = __$EvmGrantsStateCopyWithImpl;
@override @useResult
$Res call({
 List<GrantEntry> grants, bool showRevoked
});




}
/// @nodoc
class __$EvmGrantsStateCopyWithImpl<$Res>
    implements _$EvmGrantsStateCopyWith<$Res> {
  __$EvmGrantsStateCopyWithImpl(this._self, this._then);

  final _EvmGrantsState _self;
  final $Res Function(_EvmGrantsState) _then;

/// Create a copy of EvmGrantsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? grants = null,Object? showRevoked = null,}) {
  return _then(_EvmGrantsState(
grants: null == grants ? _self._grants : grants // ignore: cast_nullable_to_non_nullable
as List<GrantEntry>,showRevoked: null == showRevoked ? _self.showRevoked : showRevoked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
