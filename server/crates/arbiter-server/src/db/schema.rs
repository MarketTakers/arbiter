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
    integrity_envelope (id) {
        id -> Integer,
        entity_kind -> Text,
        entity_id -> Binary,
        payload_version -> Integer,
        key_version -> Integer,
        mac -> Binary,
        signed_at -> Integer,
        created_at -> Integer,
    }
}

diesel::table! {
    operator (id) {
        id -> Nullable<Integer>,
        share -> Binary,
        share_nonce -> Binary,
        share_salt -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    operator_identity (id) {
        id -> Integer,
        public_key -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    proposal (id) {
        id -> Integer,
        kind -> Text,
        initiator_id -> Integer,
        created_at -> Integer,
        expires_at -> Integer,
        status -> Text,
    }
}

diesel::table! {
    proposal_approve_sdk_client (proposal_id) {
        proposal_id -> Integer,
        client_id -> Integer,
    }
}

diesel::table! {
    proposal_grant_wallet_access (proposal_id) {
        proposal_id -> Integer,
        wallet_id -> Integer,
        client_id -> Integer,
    }
}

diesel::table! {
    proposal_replace_operator (proposal_id) {
        proposal_id -> Integer,
        old_operator_id -> Integer,
        new_pubkey -> Binary,
    }
}

diesel::table! {
    proposal_one_off_transaction (proposal_id) {
        proposal_id -> Integer,
        client_id -> Integer,
        wallet_address -> Binary,
        chain_id -> BigInt,
        nonce -> BigInt,
        gas_limit -> BigInt,
        max_fee_per_gas -> Binary,
        max_priority_fee_per_gas -> Binary,
        to_address -> Binary,
        value -> Binary,
        input -> Binary,
    }
}

diesel::table! {
    proposal_persistent_grant (proposal_id) {
        proposal_id -> Integer,
        wallet_access_id -> Integer,
        chain_id -> BigInt,
        valid_from -> Nullable<BigInt>,
        valid_until -> Nullable<BigInt>,
        max_gas_fee_per_gas -> Nullable<Binary>,
        max_priority_fee_per_gas -> Nullable<Binary>,
        rate_limit_count -> Nullable<Integer>,
        rate_limit_window_secs -> Nullable<BigInt>,
    }
}

diesel::table! {
    proposal_persistent_grant_ether (proposal_id) {
        proposal_id -> Integer,
        window_secs -> BigInt,
        max_volume -> Binary,
    }
}

diesel::table! {
    proposal_persistent_grant_ether_target (id) {
        id -> Integer,
        proposal_id -> Integer,
        address -> Binary,
    }
}

diesel::table! {
    proposal_persistent_grant_token (proposal_id) {
        proposal_id -> Integer,
        token_contract -> Binary,
        receiver -> Nullable<Binary>,
    }
}

diesel::table! {
    proposal_persistent_grant_token_limit (id) {
        id -> Integer,
        proposal_id -> Integer,
        window_secs -> BigInt,
        max_volume -> Binary,
    }
}

diesel::table! {
    proposal_result (proposal_id) {
        proposal_id -> Integer,
        data -> Binary,
        created_at -> Integer,
    }
}

diesel::table! {
    recovery_operator (id) {
        id -> Integer,
        share -> Binary,
        share_nonce -> Binary,
        share_salt -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    recovery_operator_identity (id) {
        id -> Integer,
        public_key -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    recovery_wakeup_request (id) {
        id -> Integer,
        requested_by -> Integer,
        requested_at -> Integer,
        cancelled_by -> Nullable<Integer>,
        cancelled_at -> Nullable<Integer>,
    }
}

diesel::table! {
    recovery_proposal_vote (id) {
        id -> Integer,
        proposal_id -> Integer,
        recovery_operator_id -> Integer,
        approve -> Bool,
        signature -> Binary,
        voted_at -> Integer,
    }
}

diesel::table! {
    proposal_vote (id) {
        id -> Integer,
        proposal_id -> Integer,
        operator_id -> Integer,
        approve -> Bool,
        signature -> Binary,
        voted_at -> Integer,
    }
}

diesel::table! {
    program_client (id) {
        id -> Integer,
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
diesel::joinable!(operator -> operator_identity (id));
diesel::joinable!(program_client -> client_metadata (metadata_id));
diesel::joinable!(proposal -> operator_identity (initiator_id));
diesel::joinable!(proposal_result -> proposal (proposal_id));
diesel::joinable!(proposal_approve_sdk_client -> proposal (proposal_id));
diesel::joinable!(proposal_grant_wallet_access -> proposal (proposal_id));
diesel::joinable!(proposal_replace_operator -> proposal (proposal_id));
diesel::joinable!(proposal_one_off_transaction -> proposal (proposal_id));
diesel::joinable!(proposal_persistent_grant -> proposal (proposal_id));
diesel::joinable!(proposal_persistent_grant_ether -> proposal_persistent_grant (proposal_id));
diesel::joinable!(proposal_persistent_grant_token -> proposal_persistent_grant (proposal_id));
diesel::joinable!(proposal_vote -> proposal (proposal_id));
diesel::joinable!(proposal_vote -> operator_identity (operator_id));
diesel::joinable!(recovery_operator -> recovery_operator_identity (id));
diesel::joinable!(recovery_proposal_vote -> proposal (proposal_id));
diesel::joinable!(recovery_proposal_vote -> recovery_operator_identity (recovery_operator_id));
diesel::joinable!(recovery_wakeup_request -> operator_identity (requested_by));

diesel::allow_tables_to_appear_in_same_query!(
    aead_encrypted,
    proposal_result,
    proposal_approve_sdk_client,
    proposal_grant_wallet_access,
    proposal_replace_operator,
    proposal_one_off_transaction,
    proposal_persistent_grant,
    proposal_persistent_grant_ether,
    proposal_persistent_grant_ether_target,
    proposal_persistent_grant_token,
    proposal_persistent_grant_token_limit,
    recovery_operator,
    recovery_operator_identity,
    recovery_wakeup_request,
    recovery_proposal_vote,
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
    integrity_envelope,
    operator,
    operator_identity,
    program_client,
    proposal,
    proposal_vote,
    root_key_history,
    tls_history,
);
