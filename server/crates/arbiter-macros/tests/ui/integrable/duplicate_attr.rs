#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "entity_a")]
#[integrable(kind = "entity_b")]
struct DuplicateAttr {
    value: i32,
}

fn main() {}
