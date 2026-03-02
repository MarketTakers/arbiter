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
    evm_basic_grant (id) {
        id -> Integer,
        wallet_id -> Integer,
        client_id -> Integer,
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
    evm_ether_transfer_log (id) {
        id -> Integer,
        grant_id -> Integer,
        client_id -> Integer,
        wallet_id -> Integer,
        chain_id -> Integer,
        recipient_address -> Binary,
        value -> Binary,
        created_at -> Integer,
    }
}

diesel::table! {
    evm_ether_transfer_volume_limit (id) {
        id -> Integer,
        grant_id -> Integer,
        window_secs -> Integer,
        max_volume -> Binary,
    }
}

diesel::table! {
    evm_token_approval_grant (id) {
        id -> Integer,
        basic_grant_id -> Integer,
        token_contract -> Binary,
        max_total_approval -> Binary,
    }
}

diesel::table! {
    evm_token_approval_grant_target (id) {
        id -> Integer,
        grant_id -> Integer,
        address -> Binary,
    }
}

diesel::table! {
    evm_token_approval_log (id) {
        id -> Integer,
        grant_id -> Integer,
        client_id -> Integer,
        wallet_id -> Integer,
        chain_id -> Integer,
        token_contract -> Binary,
        spender_address -> Binary,
        value -> Binary,
        created_at -> Integer,
    }
}

diesel::table! {
    evm_token_transfer_grant (id) {
        id -> Integer,
        basic_grant_id -> Integer,
        token_contract -> Binary,
    }
}

diesel::table! {
    evm_token_transfer_grant_target (id) {
        id -> Integer,
        grant_id -> Integer,
        address -> Binary,
    }
}

diesel::table! {
    evm_token_transfer_log (id) {
        id -> Integer,
        grant_id -> Integer,
        client_id -> Integer,
        wallet_id -> Integer,
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
    evm_unknown_call_grant (id) {
        id -> Integer,
        basic_grant_id -> Integer,
        contract -> Binary,
        selector -> Nullable<Binary>,
    }
}

diesel::table! {
    evm_unknown_call_log (id) {
        id -> Integer,
        grant_id -> Integer,
        client_id -> Integer,
        wallet_id -> Integer,
        chain_id -> Integer,
        contract -> Binary,
        selector -> Nullable<Binary>,
        call_data -> Nullable<Binary>,
        created_at -> Integer,
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
diesel::joinable!(evm_basic_grant -> evm_wallet (wallet_id));
diesel::joinable!(evm_basic_grant -> program_client (client_id));
diesel::joinable!(evm_ether_transfer_grant -> evm_basic_grant (basic_grant_id));
diesel::joinable!(evm_ether_transfer_grant_target -> evm_ether_transfer_grant (grant_id));
diesel::joinable!(evm_ether_transfer_log -> evm_ether_transfer_grant (grant_id));
diesel::joinable!(evm_ether_transfer_log -> evm_wallet (wallet_id));
diesel::joinable!(evm_ether_transfer_log -> program_client (client_id));
diesel::joinable!(evm_ether_transfer_volume_limit -> evm_ether_transfer_grant (grant_id));
diesel::joinable!(evm_token_approval_grant -> evm_basic_grant (basic_grant_id));
diesel::joinable!(evm_token_approval_grant_target -> evm_token_approval_grant (grant_id));
diesel::joinable!(evm_token_approval_log -> evm_token_approval_grant (grant_id));
diesel::joinable!(evm_token_approval_log -> evm_wallet (wallet_id));
diesel::joinable!(evm_token_approval_log -> program_client (client_id));
diesel::joinable!(evm_token_transfer_grant -> evm_basic_grant (basic_grant_id));
diesel::joinable!(evm_token_transfer_grant_target -> evm_token_transfer_grant (grant_id));
diesel::joinable!(evm_token_transfer_log -> evm_token_transfer_grant (grant_id));
diesel::joinable!(evm_token_transfer_log -> evm_wallet (wallet_id));
diesel::joinable!(evm_token_transfer_log -> program_client (client_id));
diesel::joinable!(evm_token_transfer_volume_limit -> evm_token_transfer_grant (grant_id));
diesel::joinable!(evm_unknown_call_grant -> evm_basic_grant (basic_grant_id));
diesel::joinable!(evm_unknown_call_log -> evm_unknown_call_grant (grant_id));
diesel::joinable!(evm_unknown_call_log -> evm_wallet (wallet_id));
diesel::joinable!(evm_unknown_call_log -> program_client (client_id));
diesel::joinable!(evm_wallet -> aead_encrypted (aead_encrypted_id));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    arbiter_settings,
    evm_basic_grant,
    evm_ether_transfer_grant,
    evm_ether_transfer_grant_target,
    evm_ether_transfer_log,
    evm_ether_transfer_volume_limit,
    evm_token_approval_grant,
    evm_token_approval_grant_target,
    evm_token_approval_log,
    evm_token_transfer_grant,
    evm_token_transfer_grant_target,
    evm_token_transfer_log,
    evm_token_transfer_volume_limit,
    evm_unknown_call_grant,
    evm_unknown_call_log,
    evm_wallet,
    program_client,
    root_key_history,
    tls_history,
    useragent_client,
);
