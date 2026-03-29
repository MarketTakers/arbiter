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
    client_metadata (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        version -> Nullable<Text>,
        created_at -> Integer,
    }
}

diesel::table! {
    client_metadata_history (id) {
        id -> Integer,
        metadata_id -> Integer,
        client_id -> Integer,
        created_at -> Integer,
    }
}

diesel::table! {
    evm_basic_grant (id) {
        id -> Integer,
        wallet_access_id -> Integer,
        chain_id -> Integer,
        valid_from -> Nullable<Integer>,
        valid_until -> Nullable<Integer>,
        max_gas_fee_per_gas -> Nullable<Binary>,
        max_priority_fee_per_gas -> Nullable<Binary>,
        rate_limit_count -> Nullable<Integer>,
        rate_limit_window_secs -> Nullable<Integer>,
        revoked_at -> Nullable<Integer>,
        created_at -> Integer,
    }
}

diesel::table! {
    evm_ether_transfer_grant (id) {
        id -> Integer,
        basic_grant_id -> Integer,
        limit_id -> Integer,
    }
}

diesel::table! {
    evm_ether_transfer_grant_target (id) {
        id -> Integer,
        grant_id -> Integer,
        address -> Binary,
    }
}

diesel::table! {
    evm_ether_transfer_limit (id) {
        id -> Integer,
        window_secs -> Integer,
        max_volume -> Binary,
    }
}

diesel::table! {
    evm_token_transfer_grant (id) {
        id -> Integer,
        basic_grant_id -> Integer,
        token_contract -> Binary,
        receiver -> Nullable<Binary>,
    }
}

diesel::table! {
    evm_token_transfer_log (id) {
        id -> Integer,
        grant_id -> Integer,
        log_id -> Integer,
        chain_id -> Integer,
        token_contract -> Binary,
        recipient_address -> Binary,
        value -> Binary,
        created_at -> Integer,
    }
}

diesel::table! {
    evm_token_transfer_volume_limit (id) {
        id -> Integer,
        grant_id -> Integer,
        window_secs -> Integer,
        max_volume -> Binary,
    }
}

diesel::table! {
    evm_transaction_log (id) {
        id -> Integer,
        wallet_access_id -> Integer,
        grant_id -> Integer,
        chain_id -> Integer,
        eth_value -> Binary,
        signed_at -> Integer,
    }
}

diesel::table! {
    evm_wallet (id) {
        id -> Integer,
        address -> Binary,
        aead_encrypted_id -> Integer,
        created_at -> Integer,
    }
}

diesel::table! {
    evm_wallet_access (id) {
        id -> Integer,
        wallet_id -> Integer,
        client_id -> Integer,
        created_at -> Integer,
    }
}

diesel::table! {
    program_client (id) {
        id -> Integer,
        nonce -> Integer,
        public_key -> Binary,
        metadata_id -> Integer,
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
        pubkey_integrity_tag -> Nullable<Binary>,
        key_type -> Integer,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::joinable!(aead_encrypted -> root_key_history (associated_root_key_id));
diesel::joinable!(arbiter_settings -> root_key_history (root_key_id));
diesel::joinable!(arbiter_settings -> tls_history (tls_id));
diesel::joinable!(client_metadata_history -> client_metadata (metadata_id));
diesel::joinable!(client_metadata_history -> program_client (client_id));
diesel::joinable!(evm_basic_grant -> evm_wallet_access (wallet_access_id));
diesel::joinable!(evm_ether_transfer_grant -> evm_basic_grant (basic_grant_id));
diesel::joinable!(evm_ether_transfer_grant -> evm_ether_transfer_limit (limit_id));
diesel::joinable!(evm_ether_transfer_grant_target -> evm_ether_transfer_grant (grant_id));
diesel::joinable!(evm_token_transfer_grant -> evm_basic_grant (basic_grant_id));
diesel::joinable!(evm_token_transfer_log -> evm_token_transfer_grant (grant_id));
diesel::joinable!(evm_token_transfer_log -> evm_transaction_log (log_id));
diesel::joinable!(evm_token_transfer_volume_limit -> evm_token_transfer_grant (grant_id));
diesel::joinable!(evm_transaction_log -> evm_basic_grant (grant_id));
diesel::joinable!(evm_transaction_log -> evm_wallet_access (wallet_access_id));
diesel::joinable!(evm_wallet -> aead_encrypted (aead_encrypted_id));
diesel::joinable!(evm_wallet_access -> evm_wallet (wallet_id));
diesel::joinable!(evm_wallet_access -> program_client (client_id));
diesel::joinable!(program_client -> client_metadata (metadata_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    client_metadata,
    client_metadata_history,
    evm_basic_grant,
    evm_ether_transfer_grant,
    evm_ether_transfer_grant_target,
    evm_ether_transfer_limit,
    evm_token_transfer_grant,
    evm_token_transfer_log,
    evm_token_transfer_volume_limit,
    evm_transaction_log,
    evm_wallet,
    evm_wallet_access,
    program_client,
    root_key_history,
    tls_history,
    useragent_client,
);
