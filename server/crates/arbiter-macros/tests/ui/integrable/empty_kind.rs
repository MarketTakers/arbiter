#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "")]
struct EmptyKind {
    value: i32,
}

fn main() {}
