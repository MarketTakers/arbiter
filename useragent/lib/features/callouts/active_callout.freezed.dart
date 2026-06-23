// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'active_callout.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ActiveCallout {

 String get id; String get title; String get description; String? get iconUrl; DateTime get addedAt; CalloutData get data;
/// Create a copy of ActiveCallout
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ActiveCalloutCopyWith<ActiveCallout> get copyWith => _$ActiveCalloutCopyWithImpl<ActiveCallout>(this as ActiveCallout, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ActiveCallout&&(identical(other.id, id) || other.id == id)&&(identical(other.title, title) || other.title == title)&&(identical(other.description, description) || other.description == description)&&(identical(other.iconUrl, iconUrl) || other.iconUrl == iconUrl)&&(identical(other.addedAt, addedAt) || other.addedAt == addedAt)&&(identical(other.data, data) || other.data == data));
}


@override
int get hashCode => Object.hash(runtimeType,id,title,description,iconUrl,addedAt,data);

@override
String toString() {
  return 'ActiveCallout(id: $id, title: $title, description: $description, iconUrl: $iconUrl, addedAt: $addedAt, data: $data)';
}


}

/// @nodoc
abstract mixin class $ActiveCalloutCopyWith<$Res>  {
  factory $ActiveCalloutCopyWith(ActiveCallout value, $Res Function(ActiveCallout) _then) = _$ActiveCalloutCopyWithImpl;
@useResult
$Res call({
 String id, String title, String description, String? iconUrl, DateTime addedAt, CalloutData data
});


$CalloutDataCopyWith<$Res> get data;

}
/// @nodoc
class _$ActiveCalloutCopyWithImpl<$Res>
    implements $ActiveCalloutCopyWith<$Res> {
  _$ActiveCalloutCopyWithImpl(this._self, this._then);

  final ActiveCallout _self;
  final $Res Function(ActiveCallout) _then;

/// Create a copy of ActiveCallout
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? title = null,Object? description = null,Object? iconUrl = freezed,Object? addedAt = null,Object? data = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,description: null == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as String,iconUrl: freezed == iconUrl ? _self.iconUrl : iconUrl // ignore: cast_nullable_to_non_nullable
as String?,addedAt: null == addedAt ? _self.addedAt : addedAt // ignore: cast_nullable_to_non_nullable
as DateTime,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as CalloutData,
  ));
}
/// Create a copy of ActiveCallout
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CalloutDataCopyWith<$Res> get data {
  
  return $CalloutDataCopyWith<$Res>(_self.data, (value) {
    return _then(_self.copyWith(data: value));
  });
}
}


/// Adds pattern-matching-related methods to [ActiveCallout].
extension ActiveCalloutPatterns on ActiveCallout {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ActiveCallout value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ActiveCallout() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ActiveCallout value)  $default,){
final _that = this;
switch (_that) {
case _ActiveCallout():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ActiveCallout value)?  $default,){
final _that = this;
switch (_that) {
case _ActiveCallout() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String title,  String description,  String? iconUrl,  DateTime addedAt,  CalloutData data)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ActiveCallout() when $default != null:
return $default(_that.id,_that.title,_that.description,_that.iconUrl,_that.addedAt,_that.data);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String title,  String description,  String? iconUrl,  DateTime addedAt,  CalloutData data)  $default,) {final _that = this;
switch (_that) {
case _ActiveCallout():
return $default(_that.id,_that.title,_that.description,_that.iconUrl,_that.addedAt,_that.data);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String title,  String description,  String? iconUrl,  DateTime addedAt,  CalloutData data)?  $default,) {final _that = this;
switch (_that) {
case _ActiveCallout() when $default != null:
return $default(_that.id,_that.title,_that.description,_that.iconUrl,_that.addedAt,_that.data);case _:
  return null;

}
}

}

/// @nodoc


class _ActiveCallout implements ActiveCallout {
  const _ActiveCallout({required this.id, required this.title, required this.description, this.iconUrl, required this.addedAt, required this.data});
  

@override final  String id;
@override final  String title;
@override final  String description;
@override final  String? iconUrl;
@override final  DateTime addedAt;
@override final  CalloutData data;

/// Create a copy of ActiveCallout
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ActiveCalloutCopyWith<_ActiveCallout> get copyWith => __$ActiveCalloutCopyWithImpl<_ActiveCallout>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ActiveCallout&&(identical(other.id, id) || other.id == id)&&(identical(other.title, title) || other.title == title)&&(identical(other.description, description) || other.description == description)&&(identical(other.iconUrl, iconUrl) || other.iconUrl == iconUrl)&&(identical(other.addedAt, addedAt) || other.addedAt == addedAt)&&(identical(other.data, data) || other.data == data));
}


@override
int get hashCode => Object.hash(runtimeType,id,title,description,iconUrl,addedAt,data);

@override
String toString() {
  return 'ActiveCallout(id: $id, title: $title, description: $description, iconUrl: $iconUrl, addedAt: $addedAt, data: $data)';
}


}

/// @nodoc
abstract mixin class _$ActiveCalloutCopyWith<$Res> implements $ActiveCalloutCopyWith<$Res> {
  factory _$ActiveCalloutCopyWith(_ActiveCallout value, $Res Function(_ActiveCallout) _then) = __$ActiveCalloutCopyWithImpl;
@override @useResult
$Res call({
 String id, String title, String description, String? iconUrl, DateTime addedAt, CalloutData data
});


@override $CalloutDataCopyWith<$Res> get data;

}
/// @nodoc
class __$ActiveCalloutCopyWithImpl<$Res>
    implements _$ActiveCalloutCopyWith<$Res> {
  __$ActiveCalloutCopyWithImpl(this._self, this._then);

  final _ActiveCallout _self;
  final $Res Function(_ActiveCallout) _then;

/// Create a copy of ActiveCallout
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? title = null,Object? description = null,Object? iconUrl = freezed,Object? addedAt = null,Object? data = null,}) {
  return _then(_ActiveCallout(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,description: null == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as String,iconUrl: freezed == iconUrl ? _self.iconUrl : iconUrl // ignore: cast_nullable_to_non_nullable
as String?,addedAt: null == addedAt ? _self.addedAt : addedAt // ignore: cast_nullable_to_non_nullable
as DateTime,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as CalloutData,
  ));
}

/// Create a copy of ActiveCallout
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CalloutDataCopyWith<$Res> get data {
  
  return $CalloutDataCopyWith<$Res>(_self.data, (value) {
    return _then(_self.copyWith(data: value));
  });
}
}

// dart format on
