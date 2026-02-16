// @generated automatically by Diesel CLI.

diesel::table! {
    aead_encrypted (id) {
        id -> Integer,
        current_nonce -> Integer,
        ciphertext -> Binary,
        tag -> Binary,
        schema_version -> Integer,
        argon2_salt -> Nullable<Text>,
    }
}

diesel::table! {
    arbiter_settings (id) {
        id -> Integer,
        root_key_id -> Nullable<Integer>,
        cert_key -> Binary,
        cert -> Binary,
        current_cert_id -> Nullable<Integer>,
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

diesel::table! {
    tls_certificates (id) {
        id -> Integer,
        cert -> Binary,
        cert_key -> Binary,
        not_before -> Integer,
        not_after -> Integer,
        created_at -> Integer,
        is_active -> Bool,
    }
}

diesel::table! {
    tls_rotation_state (id) {
        id -> Integer,
        state -> Text,
        new_cert_id -> Nullable<Integer>,
        initiated_at -> Nullable<Integer>,
        timeout_at -> Nullable<Integer>,
    }
}

diesel::table! {
    rotation_client_acks (rotation_id, client_key) {
        rotation_id -> Integer,
        client_key -> Text,
        ack_received_at -> Integer,
    }
}

diesel::table! {
    tls_rotation_history (id) {
        id -> Integer,
        cert_id -> Integer,
        event_type -> Text,
        timestamp -> Integer,
        details -> Nullable<Text>,
    }
}

diesel::joinable!(arbiter_settings -> aead_encrypted (root_key_id));
diesel::joinable!(arbiter_settings -> tls_certificates (current_cert_id));
diesel::joinable!(tls_rotation_state -> tls_certificates (new_cert_id));
diesel::joinable!(rotation_client_acks -> tls_certificates (rotation_id));
diesel::joinable!(tls_rotation_history -> tls_certificates (cert_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    program_client,
    useragent_client,
    tls_certificates,
    tls_rotation_state,
    rotation_client_acks,
    tls_rotation_history,
);
