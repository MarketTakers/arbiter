use hmac::digest::Digest;
use std::collections::HashSet;

/// Deterministically hash a value by feeding its fields into the hasher in a consistent order.
pub trait Hashable {
    fn hash<H: Digest>(&self, hasher: &mut H);
}

macro_rules! impl_numeric {
($($t:ty),*) => {
    $(
        impl Hashable for $t {
            fn hash<H: Digest>(&self, hasher: &mut H) {
                hasher.update(&self.to_be_bytes());
            }
        }
    )*
};
}

impl_numeric!(u8, u16, u32, u64, i8, i16, i32, i64);

impl Hashable for &[u8] {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self);
    }
}

impl Hashable for String {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self.as_bytes());
    }
}

impl<T: Hashable + PartialOrd> Hashable for Vec<T> {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        let ref_sorted = {
            let mut sorted = self.iter().collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };
        for item in ref_sorted {
            item.hash(hasher);
        }
    }
}

impl<T: Hashable + PartialOrd> Hashable for HashSet<T> {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        let ref_sorted = {
            let mut sorted = self.iter().collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };
        for item in ref_sorted {
            item.hash(hasher);
        }
    }
}

impl<T: Hashable> Hashable for Option<T> {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        match self {
            Some(value) => {
                hasher.update([1]);
                value.hash(hasher);
            }
            None => hasher.update([0]),
        }
    }
}

impl<T: Hashable> Hashable for Box<T> {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        self.as_ref().hash(hasher);
    }
}

impl<T: Hashable> Hashable for &T {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        (*self).hash(hasher);
    }
}

impl Hashable for alloy::primitives::Address {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self.as_slice());
    }
}

impl Hashable for alloy::primitives::U256 {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self.to_be_bytes::<32>());
    }
}

impl Hashable for chrono::Duration {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self.num_seconds().to_be_bytes());
    }
}

impl Hashable for chrono::DateTime<chrono::Utc> {
    fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self.timestamp_millis().to_be_bytes());
    }
}
