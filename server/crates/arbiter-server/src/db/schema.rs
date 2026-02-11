// @generated automatically by Diesel CLI.

diesel::table! {
    aead_encrypted (id) {
        id -> Integer,
        current_nonce -> Integer,
        ciphertext -> Binary,
        tag -> Binary,
        schema_version -> Integer,
    }
}

diesel::table! {
    arbiter_settings (id) {
        id -> Integer,
        root_key_id -> Nullable<Integer>,
        cert_key -> Binary,
        cert -> Binary,
    }
}

diesel::table! {
    key_identity (id) {
        id -> Integer,
        name -> Text,
        public_key -> Text,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    program_client (id) {
        id -> Integer,
        key_identity_id -> Integer,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    useragent_client (id) {
        id -> Integer,
        key_identity_id -> Integer,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::joinable!(arbiter_settings -> aead_encrypted (root_key_id));
diesel::joinable!(program_client -> key_identity (key_identity_id));
diesel::joinable!(useragent_client -> key_identity (key_identity_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    key_identity,
    program_client,
    useragent_client,
);
