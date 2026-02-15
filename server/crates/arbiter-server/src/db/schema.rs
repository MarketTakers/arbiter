// @generated automatically by Diesel CLI.

diesel::table! {
    aead_encrypted (id) {
        id -> Integer,
        current_nonce -> Binary,
        ciphertext -> Binary,
        tag -> Binary,
        schema_version -> Integer,
        created_at -> Integer,
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
    root_key_history (id) {
        id -> Integer,
        root_key_encryption_nonce -> Binary,
        data_encryption_nonce -> Binary,
        ciphertext -> Binary,
        tag -> Binary,
        schema_version -> Integer,
        salt -> Binary,
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

diesel::joinable!(arbiter_settings -> root_key_history (root_key_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    program_client,
    root_key_history,
    useragent_client,
);
