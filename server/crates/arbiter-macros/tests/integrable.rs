use arbiter_crypto::integrity::Integrable;

#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "test_entity")]
struct TestEntity {
    value: i32,
}

#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "other_entity")]
struct OtherEntity {
    label: String,
    count: u64,
}

#[test]
fn kind_is_set_correctly() {
    assert_eq!(<TestEntity as Integrable>::KIND, "test_entity");
    assert_eq!(<OtherEntity as Integrable>::KIND, "other_entity");
}

#[test]
fn version_is_positive() {
    const {
        assert!(<TestEntity as Integrable>::VERSION > 0);
        assert!(<OtherEntity as Integrable>::VERSION > 0);
    }
}

#[test]
fn different_field_layouts_produce_different_versions() {
    assert_ne!(
        <TestEntity as Integrable>::VERSION,
        <OtherEntity as Integrable>::VERSION,
    );
}

#[derive(arbiter_macros::Hashable, arbiter_macros::Integrable)]
#[integrable(kind = "generic_entity")]
struct GenericEntity<T> {
    inner: T,
}

#[test]
fn generic_struct_derives_integrable() {
    assert_eq!(
        <GenericEntity<TestEntity> as Integrable>::KIND,
        "generic_entity"
    );
    const {
        assert!(<GenericEntity<TestEntity> as Integrable>::VERSION > 0);
    }
}
