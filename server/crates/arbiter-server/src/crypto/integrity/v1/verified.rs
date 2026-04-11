use std::ops::Deref;

// todo! rewrite macro_rules to derive crate
#[macro_export]
macro_rules! VerifiedFields {
    // --- Entry point ---
    (
        $(#$attr:tt)*
        $vis:vis struct $name:ident $(<$($gen:tt),*>)?
        {
            $(
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),* $(,)?
        }
    ) => {
        // Attribute-list checks run in isolation — they only receive the attrs,
        // not the struct body.
        $crate::VerifiedFields!(@require_repr [$(#$attr)*]);
        $crate::VerifiedFields!(@reject_packed [$(#$attr)*]);

        paste::paste! {
            #[doc = concat!(
                "Field-wise verified counterpart of [`", stringify!($name), "`]."
            )]
            //
            // `#[repr(C)]` is required for the pointer casts in `inherit_ref`
            // and `inherit` to be sound. Both the source struct (enforced by
            // `@require_repr`) and this counterpart carry `#[repr(C)]`, which
            // guarantees matching field offsets. Combined with each
            // `Verified<F>` being `#[repr(transparent)]` over `F`, the two
            // structs have identical memory layout.
            //
            // `#[repr(transparent)]` is not usable here because it only permits
            // a single non-ZST field; multi-field structs would fail to compile.
            #[repr(C)]
            $vis struct [<Verified $name>] $(<$($gen),*>)?
            {
                $(
                    $field_vis $field_name : $crate::crypto::integrity::Verified<$field_ty>
                ),*
            }

            impl $(<$($gen),*>)?
                $crate::crypto::integrity::v1::verified::VerifiedFieldsAccessor
                for $crate::crypto::integrity::Verified<$name $(<$($gen),*>)?>
            {
                type Counterpart = [<Verified $name>] $(<$($gen),*>)?;

                fn inherit_ref(&self) -> &Self::Counterpart {
                    // SAFETY: `Self` is `Verified<T>` (transparent over
                    // `T #[repr(C)]`) and `Self::Counterpart` is `#[repr(C)]`
                    // with the same fields in the same order, each wrapped in
                    // a `#[repr(transparent)]` `Verified<F>`. The two types
                    // therefore have identical memory layout, which
                    // `reinterpret_layout_ref` re-checks as size/align
                    // equality at monomorphization.
                    unsafe {
                        $crate::crypto::integrity::v1::verified::reinterpret_layout_ref::<
                            Self,
                            Self::Counterpart,
                        >(self)
                    }
                }

                fn inherit(self) -> Self::Counterpart {
                    // SAFETY: identical layout — see `inherit_ref`. The owned
                    // helper additionally suppresses the source destructor so
                    // the returned counterpart owns the original bytes (no
                    // double-drop is possible).
                    unsafe {
                        $crate::crypto::integrity::v1::verified::reinterpret_layout::<
                            Self,
                            Self::Counterpart,
                        >(self)
                    }
                }
            }
        }
    };

    // --- @require_repr: ensure `#[repr(C)]` appears in the attribute list ---
    (@require_repr [#[repr(C)] $($rest:tt)*]) => {};
    (@require_repr [#$other:tt $($rest:tt)*]) => {
        $crate::VerifiedFields!(@require_repr [$($rest)*]);
    };
    (@require_repr []) => {
        ::std::compile_error!(
            "VerifiedFields requires `#[repr(C)]` on the struct to guarantee field layout"
        );
    };

    // --- @reject_packed: walk attrs and reject any `#[repr(..., packed, ...)]`.
    //
    // Without this, a packed struct would still fail at monomorphization via
    // the const assertions inside the `reinterpret_layout*` helpers, but the
    // diagnostic would be much harder to read. `align(N)` is *not* rejected
    // here because const assertions catch alignment mismatches cleanly, and
    // forbidding it would be unnecessarily restrictive.
    (@reject_packed [#[repr($($inner:tt)*)] $($rest:tt)*]) => {
        $crate::VerifiedFields!(@reject_packed_inner [$($inner)*]);
        $crate::VerifiedFields!(@reject_packed [$($rest)*]);
    };
    (@reject_packed [#$other:tt $($rest:tt)*]) => {
        $crate::VerifiedFields!(@reject_packed [$($rest)*]);
    };
    (@reject_packed []) => {};

    (@reject_packed_inner [packed $($rest:tt)*]) => {
        ::std::compile_error!(
            "VerifiedFields does not support packed layouts; the generated \
             counterpart would not share layout with the source struct"
        );
    };
    (@reject_packed_inner [$first:tt $($rest:tt)*]) => {
        $crate::VerifiedFields!(@reject_packed_inner [$($rest)*]);
    };
    (@reject_packed_inner []) => {};
}

/// Implemented on `Verified<T>` by [`VerifiedFields!`], exposing the field-wise counterpart.
///
/// ## Disclaimer
/// Do not implement this trait manually. It is intended to be implemented only
/// by the `VerifiedFields!` macro, which generates the necessary layout
/// guarantees for sound pointer casts.
///
/// ## Soundness
/// When [`verify_entity`][crate::crypto::integrity::verify_entity] attests an
/// entity, it returns `Verified<T>` — an aggregate proof over the whole value.
/// This trait converts that wrapper into `Counterpart` (e.g.
/// `VerifiedMyStruct`), where every field is individually wrapped in
/// [`Verified`], allowing verified data to flow into functions that require
/// `Verified<FieldType>` without re-verifying.
///
/// ## Safety
/// The conversion is a zero-cost reinterpretation — no copying (beyond a
/// bitwise move in the owned variant) or HMAC work occurs. Soundness rests on
/// identical memory layout between `Verified<T>` and `Counterpart`:
///
/// - `T` carries `#[repr(C)]` (enforced by `@require_repr` in the macro).
/// - `T` does **not** carry `packed` (enforced by `@reject_packed`).
/// - `Counterpart` also carries `#[repr(C)]`, with the same fields in the same
///   order.
/// - Each `Verified<F>` field is `#[repr(transparent)]` over `F`, so its size
///   and alignment match `F` exactly.
/// - `Verified<T>` itself is `#[repr(transparent)]` over `T`.
///
/// As an additional machine-checked guard, [`reinterpret_layout`] and
/// [`reinterpret_layout_ref`] assert size/align equality of the two types at
/// monomorphization time.
///
/// The trait is implemented directly on `Verified<T>` (not on `T`), so no
/// `Deref`-coercion or auto-ref stripping is needed at call sites — the impl
/// is unambiguous.
pub trait VerifiedFieldsAccessor {
    /// The field-wise verified counterpart, e.g. `VerifiedMyStruct`.
    type Counterpart;

    /// Reinterprets `&self` as `&Counterpart` via a layout-preserving pointer cast.
    ///
    /// No data is copied and no re-verification occurs. The returned reference
    /// borrows from `self` and has the same lifetime.
    fn inherit_ref(&self) -> &Self::Counterpart;

    /// Consumes `self` and returns `Counterpart` via a layout-preserving
    /// bitwise move.
    ///
    /// The original `Verified<T>` is moved without running its destructor
    /// (there is none — `Verified` is a transparent wrapper with no heap
    /// allocation), and the returned counterpart owns the original bytes. No
    /// re-verification occurs.
    fn inherit(self) -> Self::Counterpart;
}

/// A value whose integrity has been verified against the HMAC envelope stored
/// in the database.
///
/// `Verified<T>` is a zero-cost transparent wrapper produced exclusively by
/// [`crate::crypto::integrity`](super) module's functions. Holding one is proof
/// that the underlying value passed an HMAC check keyed with the vault's
/// integrity subkey.
///
/// The wrapper is intentionally narrow: it does not expose a constructor and
/// the inner value cannot be moved out without explicitly calling
/// [`drop_verification_provenance`][Verified::drop_verification_provenance],
/// making accidental provenance loss visible at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[must_use = "Verified<T> is a proof-bearing wrapper; use self.drop_verification_provenance() to explicitly discard integrity provenance when needed"]
pub struct Verified<T>(T);

impl<T> AsRef<Verified<T>> for Verified<&T> {
    fn as_ref(&self) -> &Verified<T> {
        // SAFETY: `Verified<T>` is `#[repr(transparent)]` over `T`, so `&T`
        // and `&Verified<T>` have identical layout.
        unsafe { reinterpret_layout_ref::<T, Verified<T>>(self.0) }
    }
}

impl<T> Deref for Verified<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Verified<T> {
    /// Unwraps the verified value, discarding the integrity provenance.
    ///
    /// The name is intentionally verbose — call sites where provenance is
    /// dropped should be easy to find and audit.
    pub fn drop_verification_provenance(self) -> T {
        self.0
    }

    /// Constructs a `Verified<T>` by wrapping a `T`.
    pub(super) fn new(value: T) -> Self {
        Self(value)
    }

    /// Constructs a `Verified<T>` from a raw value without performing any
    /// integrity check. Only available in test builds; use the integrity
    /// module's functions to obtain a `Verified<T>` in production code.
    #[cfg(test)]
    pub(crate) fn new_unchecked(value: T) -> Self {
        Self(value)
    }

    /// Reinterprets `&T` as `&Verified<T>`.
    #[allow(dead_code)]
    pub(super) fn from_ref(from: &T) -> &Self {
        // SAFETY: `Self` is `#[repr(transparent)]` over `T`.
        unsafe { reinterpret_layout_ref::<T, Self>(from) }
    }
}

/// Bit-copies `value: From` into a `To`, suppressing the source destructor so
/// the destination owns the bytes.
///
/// # Safety
///
/// The caller must guarantee that `From` and `To` have identical in-memory
/// layout — the raw bytes that encode a valid `From` must also encode a valid
/// `To`.
///
/// A `union` is used instead of [`std::mem::transmute`] because `transmute`
/// rejects generic source/destination types at the call site even when their
/// sizes are provably equal at monomorphization time.
#[allow(dead_code)]
#[inline]
pub const unsafe fn reinterpret_layout<From, To>(value: From) -> To {
    const {
        assert!(
            ::std::mem::size_of::<From>() == ::std::mem::size_of::<To>(),
            "reinterpret_layout: source and destination must have identical size"
        );
        assert!(
            ::std::mem::align_of::<From>() == ::std::mem::align_of::<To>(),
            "reinterpret_layout: source and destination must have identical alignment"
        );
    }
    union Reinterpret<A, B> {
        from: ::std::mem::ManuallyDrop<A>,
        to: ::std::mem::ManuallyDrop<B>,
    }
    // SAFETY: caller guarantees layout equivalence (see fn docs). The union
    // write-read copies the raw bytes of `value` into a `To` slot, and
    // `ManuallyDrop` on the source side suppresses its destructor so the
    // destination owns the bytes unambiguously — no double-drop is possible.
    unsafe {
        ::std::mem::ManuallyDrop::into_inner(
            Reinterpret {
                from: ::std::mem::ManuallyDrop::new(value),
            }
            .to,
        )
    }
}

/// Reinterprets `&From` as `&To` via a layout-preserving pointer cast.
///
/// # Safety
///
/// Same invariants as [`reinterpret_layout`].
#[inline]
pub const unsafe fn reinterpret_layout_ref<From, To>(value: &From) -> &To {
    const {
        assert!(
            ::std::mem::size_of::<From>() == ::std::mem::size_of::<To>(),
            "reinterpret_layout_ref: source and destination must have identical size"
        );
        assert!(
            ::std::mem::align_of::<From>() == ::std::mem::align_of::<To>(),
            "reinterpret_layout_ref: source and destination must have identical alignment"
        );
    }
    // SAFETY: caller guarantees layout equivalence (see fn docs). A reference
    // cast between identically-laid-out types produces a reference with the
    // same address and lifetime, which is sound.
    unsafe { &*(value as *const From as *const To) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(VerifiedFields!)]
    #[repr(C)]
    #[derive(Default, Clone)]
    pub struct MyStruct<T> {
        pub field1: String,
        pub field2: T,
    }

    fn verify<T>(t: T) -> Verified<T> {
        Verified(t)
    }

    // --- inherit_ref ---

    // Verifies that `inherit_ref` returns a reference to the same memory
    // address, confirming that no copy is made and the cast is purely a
    // reinterpretation.
    #[test]
    fn inherit_ref_is_same_address() {
        let v = verify(MyStruct {
            field1: "hello".into(),
            field2: 42u32,
        });
        let fields = v.inherit_ref();
        assert_eq!(
            &v as *const _ as *const u8, fields as *const _ as *const u8,
            "inherit_ref must return a pointer to the same memory, not a copy"
        );
    }

    // Verifies that field values are correctly accessible after `inherit_ref`.
    #[test]
    fn inherit_ref_field_values() {
        let v = verify(MyStruct {
            field1: "hello".into(),
            field2: 99u32,
        });
        let fields = v.inherit_ref();
        assert_eq!(*fields.field1, "hello");
        assert_eq!(*fields.field2, 99u32);
    }

    // Verifies that casting the counterpart back to `Verified<T>` via a raw
    // pointer lands on the original address — confirms the round-trip is a
    // pure reinterpretation.
    #[test]
    fn inherit_ref_cast_roundtrip() {
        let v = verify(MyStruct {
            field1: "x".into(),
            field2: 7u32,
        });
        let fields: &VerifiedMyStruct<u32> = v.inherit_ref();
        let back_ptr = fields as *const VerifiedMyStruct<u32> as *const Verified<MyStruct<u32>>;
        assert_eq!(
            back_ptr as *const u8, &v as *const _ as *const u8,
            "cast of counterpart must point back to the same Verified<T>"
        );
    }

    // ZST fields must still produce a counterpart with identical layout — the
    // const asserts in `reinterpret_layout_ref` guard this at monomorphization.
    #[test]
    fn inherit_ref_with_zst_field() {
        #[derive(VerifiedFields!)]
        #[repr(C)]
        struct WithZst {
            pub unit: (),
            pub val: u64,
        }

        let v = Verified(WithZst { unit: (), val: 777 });
        let fields = v.inherit_ref();
        assert_eq!(*fields.val, 777);
        assert_eq!(*fields.unit, ());
    }

    // --- inherit ---

    // Verifies that `inherit` preserves field values in the owned counterpart.
    #[test]
    fn inherit_field_values() {
        let v = verify(MyStruct {
            field1: "world".into(),
            field2: 1234u64,
        });
        let VerifiedMyStruct { field1, field2 } = v.inherit();
        assert_eq!(*field1, "world");
        assert_eq!(*field2, 1234u64);
    }

    // Verifies that `inherit` does not double-drop the inner value.
    // If `ManuallyDrop` handling is wrong, running under Miri or with a drop
    // counter catches a double-free.
    #[test]
    fn inherit_no_double_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;
        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }

        #[derive(VerifiedFields!)]
        #[repr(C)]
        struct WithDrop {
            pub val: DropCounter,
        }

        DROP_COUNT.store(0, Ordering::Relaxed);
        {
            let v = Verified(WithDrop { val: DropCounter });
            let _ = v.inherit();
        }
        assert_eq!(
            DROP_COUNT.load(Ordering::Relaxed),
            1,
            "DropCounter must be dropped exactly once"
        );
    }

    // --- Verified::from_ref ---

    #[test]
    fn from_ref_is_same_address() {
        let val = 42u32;
        let verified: &Verified<u32> = Verified::from_ref(&val);
        assert_eq!(
            &val as *const u32 as *const u8, verified as *const _ as *const u8,
            "from_ref must alias the original reference, not copy the value"
        );
    }

    #[test]
    fn from_ref_value_preserved() {
        let val = String::from("test");
        let verified: &Verified<String> = Verified::from_ref(&val);
        assert_eq!(**verified, "test");
    }

    // --- AsRef<Verified<T>> for Verified<&T> ---

    #[test]
    fn verified_ref_as_ref_is_same_address() {
        let val = 99u32;
        let vref: Verified<&u32> = Verified(&val);
        let v: &Verified<u32> = vref.as_ref();
        assert_eq!(
            &val as *const u32 as *const u8, v as *const _ as *const u8,
            "AsRef<Verified<T>> for Verified<&T> must alias the referent, not copy it"
        );
    }
}
