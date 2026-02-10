// @generated automatically by Diesel CLI.

diesel::table! {
    arbiter_settings (rowid) {
        rowid -> Integer,
        root_key_enc -> Nullable<Binary>,
        cert_key -> Binary,
        cert -> Binary,
    }
}

diesel::table! {
    key_identity (id) {
        id -> Nullable<Integer>,
        name -> Text,
        public_key -> Text,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    program_client (id) {
        id -> Nullable<Integer>,
        key_identity_id -> Integer,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    useragent_client (id) {
        id -> Nullable<Integer>,
        key_identity_id -> Integer,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::joinable!(program_client -> key_identity (key_identity_id));
diesel::joinable!(useragent_client -> key_identity (key_identity_id));

diesel::allow_tables_to_appear_in_same_query!(
    arbiter_settings,
    key_identity,
    program_client,
    useragent_client,
);
