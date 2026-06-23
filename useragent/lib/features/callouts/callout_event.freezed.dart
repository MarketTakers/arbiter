// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'callout_event.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CalloutData {

 String get pubkey; ClientInfo get clientInfo;
/// Create a copy of CalloutData
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalloutDataCopyWith<CalloutData> get copyWith => _$CalloutDataCopyWithImpl<CalloutData>(this as CalloutData, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalloutData&&(identical(other.pubkey, pubkey) || other.pubkey == pubkey)&&(identical(other.clientInfo, clientInfo) || other.clientInfo == clientInfo));
}


@override
int get hashCode => Object.hash(runtimeType,pubkey,clientInfo);

@override
String toString() {
  return 'CalloutData(pubkey: $pubkey, clientInfo: $clientInfo)';
}


}

/// @nodoc
abstract mixin class $CalloutDataCopyWith<$Res>  {
  factory $CalloutDataCopyWith(CalloutData value, $Res Function(CalloutData) _then) = _$CalloutDataCopyWithImpl;
@useResult
$Res call({
 String pubkey, ClientInfo clientInfo
});




}
/// @nodoc
class _$CalloutDataCopyWithImpl<$Res>
    implements $CalloutDataCopyWith<$Res> {
  _$CalloutDataCopyWithImpl(this._self, this._then);

  final CalloutData _self;
  final $Res Function(CalloutData) _then;

/// Create a copy of CalloutData
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? pubkey = null,Object? clientInfo = null,}) {
  return _then(_self.copyWith(
pubkey: null == pubkey ? _self.pubkey : pubkey // ignore: cast_nullable_to_non_nullable
as String,clientInfo: null == clientInfo ? _self.clientInfo : clientInfo // ignore: cast_nullable_to_non_nullable
as ClientInfo,
  ));
}

}


/// Adds pattern-matching-related methods to [CalloutData].
extension CalloutDataPatterns on CalloutData {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ConnectApprovalData value)?  connectApproval,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ConnectApprovalData() when connectApproval != null:
return connectApproval(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ConnectApprovalData value)  connectApproval,}){
final _that = this;
switch (_that) {
case ConnectApprovalData():
return connectApproval(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ConnectApprovalData value)?  connectApproval,}){
final _that = this;
switch (_that) {
case ConnectApprovalData() when connectApproval != null:
return connectApproval(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String pubkey,  ClientInfo clientInfo)?  connectApproval,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ConnectApprovalData() when connectApproval != null:
return connectApproval(_that.pubkey,_that.clientInfo);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String pubkey,  ClientInfo clientInfo)  connectApproval,}) {final _that = this;
switch (_that) {
case ConnectApprovalData():
return connectApproval(_that.pubkey,_that.clientInfo);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String pubkey,  ClientInfo clientInfo)?  connectApproval,}) {final _that = this;
switch (_that) {
case ConnectApprovalData() when connectApproval != null:
return connectApproval(_that.pubkey,_that.clientInfo);case _:
  return null;

}
}

}

/// @nodoc


class ConnectApprovalData implements CalloutData {
  const ConnectApprovalData({required this.pubkey, required this.clientInfo});
  

@override final  String pubkey;
@override final  ClientInfo clientInfo;

/// Create a copy of CalloutData
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ConnectApprovalDataCopyWith<ConnectApprovalData> get copyWith => _$ConnectApprovalDataCopyWithImpl<ConnectApprovalData>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectApprovalData&&(identical(other.pubkey, pubkey) || other.pubkey == pubkey)&&(identical(other.clientInfo, clientInfo) || other.clientInfo == clientInfo));
}


@override
int get hashCode => Object.hash(runtimeType,pubkey,clientInfo);

@override
String toString() {
  return 'CalloutData.connectApproval(pubkey: $pubkey, clientInfo: $clientInfo)';
}


}

/// @nodoc
abstract mixin class $ConnectApprovalDataCopyWith<$Res> implements $CalloutDataCopyWith<$Res> {
  factory $ConnectApprovalDataCopyWith(ConnectApprovalData value, $Res Function(ConnectApprovalData) _then) = _$ConnectApprovalDataCopyWithImpl;
@override @useResult
$Res call({
 String pubkey, ClientInfo clientInfo
});




}
/// @nodoc
class _$ConnectApprovalDataCopyWithImpl<$Res>
    implements $ConnectApprovalDataCopyWith<$Res> {
  _$ConnectApprovalDataCopyWithImpl(this._self, this._then);

  final ConnectApprovalData _self;
  final $Res Function(ConnectApprovalData) _then;

/// Create a copy of CalloutData
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? pubkey = null,Object? clientInfo = null,}) {
  return _then(ConnectApprovalData(
pubkey: null == pubkey ? _self.pubkey : pubkey // ignore: cast_nullable_to_non_nullable
as String,clientInfo: null == clientInfo ? _self.clientInfo : clientInfo // ignore: cast_nullable_to_non_nullable
as ClientInfo,
  ));
}


}

/// @nodoc
mixin _$CalloutEvent {

 String get id;
/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalloutEventCopyWith<CalloutEvent> get copyWith => _$CalloutEventCopyWithImpl<CalloutEvent>(this as CalloutEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalloutEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'CalloutEvent(id: $id)';
}


}

/// @nodoc
abstract mixin class $CalloutEventCopyWith<$Res>  {
  factory $CalloutEventCopyWith(CalloutEvent value, $Res Function(CalloutEvent) _then) = _$CalloutEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$CalloutEventCopyWithImpl<$Res>
    implements $CalloutEventCopyWith<$Res> {
  _$CalloutEventCopyWithImpl(this._self, this._then);

  final CalloutEvent _self;
  final $Res Function(CalloutEvent) _then;

/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [CalloutEvent].
extension CalloutEventPatterns on CalloutEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CalloutEventAdded value)?  added,TResult Function( CalloutEventCancelled value)?  cancelled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CalloutEventAdded() when added != null:
return added(_that);case CalloutEventCancelled() when cancelled != null:
return cancelled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CalloutEventAdded value)  added,required TResult Function( CalloutEventCancelled value)  cancelled,}){
final _that = this;
switch (_that) {
case CalloutEventAdded():
return added(_that);case CalloutEventCancelled():
return cancelled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CalloutEventAdded value)?  added,TResult? Function( CalloutEventCancelled value)?  cancelled,}){
final _that = this;
switch (_that) {
case CalloutEventAdded() when added != null:
return added(_that);case CalloutEventCancelled() when cancelled != null:
return cancelled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id,  CalloutData data)?  added,TResult Function( String id)?  cancelled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CalloutEventAdded() when added != null:
return added(_that.id,_that.data);case CalloutEventCancelled() when cancelled != null:
return cancelled(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id,  CalloutData data)  added,required TResult Function( String id)  cancelled,}) {final _that = this;
switch (_that) {
case CalloutEventAdded():
return added(_that.id,_that.data);case CalloutEventCancelled():
return cancelled(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id,  CalloutData data)?  added,TResult? Function( String id)?  cancelled,}) {final _that = this;
switch (_that) {
case CalloutEventAdded() when added != null:
return added(_that.id,_that.data);case CalloutEventCancelled() when cancelled != null:
return cancelled(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class CalloutEventAdded implements CalloutEvent {
  const CalloutEventAdded({required this.id, required this.data});
  

@override final  String id;
 final  CalloutData data;

/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalloutEventAddedCopyWith<CalloutEventAdded> get copyWith => _$CalloutEventAddedCopyWithImpl<CalloutEventAdded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalloutEventAdded&&(identical(other.id, id) || other.id == id)&&(identical(other.data, data) || other.data == data));
}


@override
int get hashCode => Object.hash(runtimeType,id,data);

@override
String toString() {
  return 'CalloutEvent.added(id: $id, data: $data)';
}


}

/// @nodoc
abstract mixin class $CalloutEventAddedCopyWith<$Res> implements $CalloutEventCopyWith<$Res> {
  factory $CalloutEventAddedCopyWith(CalloutEventAdded value, $Res Function(CalloutEventAdded) _then) = _$CalloutEventAddedCopyWithImpl;
@override @useResult
$Res call({
 String id, CalloutData data
});


$CalloutDataCopyWith<$Res> get data;

}
/// @nodoc
class _$CalloutEventAddedCopyWithImpl<$Res>
    implements $CalloutEventAddedCopyWith<$Res> {
  _$CalloutEventAddedCopyWithImpl(this._self, this._then);

  final CalloutEventAdded _self;
  final $Res Function(CalloutEventAdded) _then;

/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? data = null,}) {
  return _then(CalloutEventAdded(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as CalloutData,
  ));
}

/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CalloutDataCopyWith<$Res> get data {
  
  return $CalloutDataCopyWith<$Res>(_self.data, (value) {
    return _then(_self.copyWith(data: value));
  });
}
}

/// @nodoc


class CalloutEventCancelled implements CalloutEvent {
  const CalloutEventCancelled({required this.id});
  

@override final  String id;

/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalloutEventCancelledCopyWith<CalloutEventCancelled> get copyWith => _$CalloutEventCancelledCopyWithImpl<CalloutEventCancelled>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalloutEventCancelled&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'CalloutEvent.cancelled(id: $id)';
}


}

/// @nodoc
abstract mixin class $CalloutEventCancelledCopyWith<$Res> implements $CalloutEventCopyWith<$Res> {
  factory $CalloutEventCancelledCopyWith(CalloutEventCancelled value, $Res Function(CalloutEventCancelled) _then) = _$CalloutEventCancelledCopyWithImpl;
@override @useResult
$Res call({
 String id
});




}
/// @nodoc
class _$CalloutEventCancelledCopyWithImpl<$Res>
    implements $CalloutEventCancelledCopyWith<$Res> {
  _$CalloutEventCancelledCopyWithImpl(this._self, this._then);

  final CalloutEventCancelled _self;
  final $Res Function(CalloutEventCancelled) _then;

/// Create a copy of CalloutEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(CalloutEventCancelled(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
