#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "bad kind!")]
struct InvalidKind {
    value: i32,
}

fn main() {}
