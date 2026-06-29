use crate::hashing::Hashable;

pub trait Integrable: Hashable {
    const KIND: &'static str;
    const VERSION: i32 = 1;
}
