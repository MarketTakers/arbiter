#[derive(arbiter_macros::Integrable)]
#[integrable(kind = "my_enum")]
enum MyEnum {
    A,
    B,
}

fn main() {}
