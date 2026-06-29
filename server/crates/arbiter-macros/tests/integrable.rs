use arbiter_crypto::integrity::Integrable;

#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "test_entity")]
struct TestEntity {
    value: i32,
}

#[test]
fn default_version_is_one() {
    assert_eq!(<TestEntity as Integrable>::VERSION, 1, "default version must be 1");
    assert_eq!(<TestEntity as Integrable>::KIND, "test_entity");
}

#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "versioned_entity", version = 3)]
struct VersionedEntity {
    data: String,
}

#[test]
fn explicit_version_attribute() {
    assert_eq!(<VersionedEntity as Integrable>::VERSION, 3);
    assert_eq!(<VersionedEntity as Integrable>::KIND, "versioned_entity");
}
