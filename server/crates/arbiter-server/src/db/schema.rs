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
    program_client (id) {
        id -> Integer,
        nonce -> Integer,
        public_key -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    useragent_client (id) {
        id -> Integer,
        nonce -> Integer,
        public_key -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::joinable!(arbiter_settings -> aead_encrypted (root_key_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    program_client,
    useragent_client,
);
