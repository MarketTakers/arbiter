// @generated automatically by Diesel CLI.

diesel::table! {
    aead_encrypted (id) {
        id -> Integer,
        current_nonce -> Binary,
        ciphertext -> Binary,
        tag -> Binary,
        schema_version -> Integer,
        associated_root_key_id -> Integer,
        created_at -> Integer,
    }
}

diesel::table! {
    arbiter_settings (id) {
        id -> Integer,
        root_key_id -> Nullable<Integer>,
        tls_id -> Nullable<Integer>,
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
    tls_history (id) {
        id -> Integer,
        cert -> Text,
        cert_key -> Text,
        ca_cert -> Text,
        ca_key -> Text,
        created_at -> Integer,
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

diesel::joinable!(aead_encrypted -> root_key_history (associated_root_key_id));
diesel::joinable!(arbiter_settings -> root_key_history (root_key_id));
diesel::joinable!(arbiter_settings -> tls_history (tls_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    program_client,
    root_key_history,
    tls_history,
    useragent_client,
);
