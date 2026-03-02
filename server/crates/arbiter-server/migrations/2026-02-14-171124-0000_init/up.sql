create table if not exists root_key_history (
    id INTEGER not null PRIMARY KEY,
    -- root key stored as aead encrypted artifact, with only difference that it's decrypted by unseal key (derived from user password)
    root_key_encryption_nonce blob not null default(1), -- if re-encrypted, this should be incremented. Used for encrypting root key
    data_encryption_nonce blob not null default(1), -- nonce used for encrypting with key itself
    ciphertext blob not null,
    tag blob not null,
    schema_version integer not null default(1), -- server would need to reencrypt, because this means that we have changed algorithm
    salt blob not null -- for key deriviation 
) STRICT;

create table if not exists aead_encrypted (
    id INTEGER not null PRIMARY KEY,
    current_nonce blob not null default(1), -- if re-encrypted, this should be incremented
    ciphertext blob not null,
    tag blob not null,
    schema_version integer not null default(1), -- server would need to reencrypt, because this means that we have changed algorithm
    associated_root_key_id integer not null references root_key_history (id) on delete RESTRICT,
    created_at integer not null default(unixepoch ('now'))
) STRICT;

create unique index if not exists uniq_nonce_per_root_key on aead_encrypted (
    current_nonce,
    associated_root_key_id
);

create table if not exists tls_history (
    id INTEGER not null PRIMARY KEY,
    cert text not null,
    cert_key text not null, -- PEM Encoded private key
    ca_cert text not null, 
    ca_key text not null, -- PEM Encoded private key
    created_at integer not null default(unixepoch ('now'))
) STRICT;

-- This is a singleton
create table if not exists arbiter_settings (
    id INTEGER not null PRIMARY KEY CHECK (id = 1), -- singleton row, id must be 1
    root_key_id integer references root_key_history (id) on delete RESTRICT, -- if null, means wasn't bootstrapped yet
    tls_id integer references tls_history (id) on delete RESTRICT
) STRICT;

insert into arbiter_settings (id) values (1) on conflict do nothing; -- ensure singleton row exists

create table if not exists useragent_client (
    id integer not null primary key,
    nonce integer not null default(1), -- used for auth challenge
    public_key blob not null,
    created_at integer not null default(unixepoch ('now')),
    updated_at integer not null default(unixepoch ('now'))
) STRICT;

create table if not exists program_client (
    id integer not null primary key,
    nonce integer not null default(1), -- used for auth challenge
    public_key blob not null,
    created_at integer not null default(unixepoch ('now')),
    updated_at integer not null default(unixepoch ('now'))
) STRICT;

create table if not exists evm_wallet (
    id integer not null primary key,
    address blob not null, -- 20-byte Ethereum address
    aead_encrypted_id integer not null references aead_encrypted (id) on delete RESTRICT,
    created_at integer not null default(unixepoch ('now'))
) STRICT;

create unique index if not exists uniq_evm_wallet_address on evm_wallet (address);
create unique index if not exists uniq_evm_wallet_aead on evm_wallet (aead_encrypted_id);

-- Shared grant properties: client scope, timeframe, fee caps, and rate limit
create table if not exists evm_basic_grant (
    id integer not null primary key,
    wallet_id integer not null references evm_wallet(id) on delete restrict, 
    client_id integer not null references program_client(id) on delete restrict,
    chain_id integer not null,              -- EIP-155 chain ID
    valid_from integer,                     -- unix timestamp (seconds), null = no lower bound
    valid_until integer,                    -- unix timestamp (seconds), null = no upper bound
    max_gas_fee_per_gas blob,               -- big-endian 32-byte U256, null = unlimited
    max_priority_fee_per_gas blob,          -- big-endian 32-byte U256, null = unlimited
    rate_limit_count integer,               -- max transactions in window, null = unlimited
    rate_limit_window_secs integer,         -- window duration in seconds, null = unlimited
    revoked_at integer,                     -- unix timestamp when revoked, null = still active
    created_at integer not null default(unixepoch('now'))
) STRICT;

create index if not exists idx_evm_basic_grant_wallet_chain on evm_basic_grant(client_id, wallet_id, chain_id);

-- ERC20 token transfer grant
create table if not exists evm_token_transfer_grant (
    id integer not null primary key,
    basic_grant_id integer not null unique references evm_basic_grant(id) on delete cascade,
    token_contract blob not null            -- 20-byte ERC20 contract address
) STRICT;

-- Specific recipient addresses for a token transfer grant (only used when target_all = 0)
create table if not exists evm_token_transfer_grant_target (
    id integer not null primary key,
    grant_id integer not null references evm_token_transfer_grant(id) on delete cascade,
    address blob not null                   -- 20-byte recipient address
) STRICT;

create unique index if not exists uniq_token_transfer_target on evm_token_transfer_grant_target(grant_id, address);

-- Per-window volume limits for token transfer grants
create table if not exists evm_token_transfer_volume_limit (
    id integer not null primary key,
    grant_id integer not null references evm_token_transfer_grant(id) on delete cascade,
    window_secs integer not null,           -- window duration in seconds
    max_volume blob not null                -- big-endian 32-byte U256
) STRICT;

-- ERC20 token approval grant
create table if not exists evm_token_approval_grant (
    id integer not null primary key,
    basic_grant_id integer not null unique references evm_basic_grant(id) on delete cascade,
    token_contract blob not null,           -- 20-byte ERC20 contract address
    max_total_approval blob not null        -- big-endian 32-byte U256; max cumulative approval value
) STRICT;

-- Specific spender addresses for a token approval grant (only used when target_all = 0)
create table if not exists evm_token_approval_grant_target (
    id integer not null primary key,
    grant_id integer not null references evm_token_approval_grant(id) on delete cascade,
    address blob not null                   -- 20-byte spender address
) STRICT;

create unique index if not exists uniq_token_approval_target on evm_token_approval_grant_target(grant_id, address);

-- Plain ether transfer grant
create table if not exists evm_ether_transfer_grant (
    id integer not null primary key,
    basic_grant_id integer not null unique references evm_basic_grant(id) on delete cascade
) STRICT;

-- Specific recipient addresses for an ether transfer grant (only used when target_all = 0)
create table if not exists evm_ether_transfer_grant_target (
    id integer not null primary key,
    grant_id integer not null references evm_ether_transfer_grant(id) on delete cascade,
    address blob not null                   -- 20-byte recipient address
) STRICT;

create unique index if not exists uniq_ether_transfer_target on evm_ether_transfer_grant_target(grant_id, address);

-- Per-window volume limits for ether transfer grants
create table if not exists evm_ether_transfer_volume_limit (
    id integer not null primary key,
    grant_id integer not null references evm_ether_transfer_grant(id) on delete cascade,
    window_secs integer not null,
    max_volume blob not null                -- big-endian 32-byte U256
) STRICT;

-- Unknown / opaque contract call grant
create table if not exists evm_unknown_call_grant (
    id integer not null primary key,
    basic_grant_id integer not null unique references evm_basic_grant(id) on delete cascade,
    contract blob not null,                 -- 20-byte target contract address
    selector blob                           -- 4-byte function selector, null = allow any selector
) STRICT;

-- Log table for ether transfer grant usage
create table if not exists evm_ether_transfer_log (
    id integer not null primary key,
    grant_id integer not null references evm_ether_transfer_grant(id) on delete restrict,
    client_id integer not null references program_client(id) on delete restrict,
    wallet_id integer not null references evm_wallet(id) on delete restrict,
    chain_id integer not null,              -- EIP-155 chain ID
    recipient_address blob not null,        -- 20-byte recipient address
    value blob not null,                    -- big-endian 32-byte U256
    created_at integer not null default(unixepoch('now'))
) STRICT;

create index if not exists idx_ether_transfer_log_grant on evm_ether_transfer_log(grant_id);
create index if not exists idx_ether_transfer_log_client on evm_ether_transfer_log(client_id);
create index if not exists idx_ether_transfer_log_wallet on evm_ether_transfer_log(wallet_id);

-- Log table for token transfer grant usage
create table if not exists evm_token_transfer_log (
    id integer not null primary key,
    grant_id integer not null references evm_token_transfer_grant(id) on delete restrict,
    client_id integer not null references program_client(id) on delete restrict,
    wallet_id integer not null references evm_wallet(id) on delete restrict,
    chain_id integer not null,              -- EIP-155 chain ID
    token_contract blob not null,           -- 20-byte ERC20 contract address
    recipient_address blob not null,        -- 20-byte recipient address
    value blob not null,                    -- big-endian 32-byte U256
    created_at integer not null default(unixepoch('now'))
) STRICT;

create index if not exists idx_token_transfer_log_grant on evm_token_transfer_log(grant_id);
create index if not exists idx_token_transfer_log_client on evm_token_transfer_log(client_id);
create index if not exists idx_token_transfer_log_wallet on evm_token_transfer_log(wallet_id);

-- Log table for token approval grant usage
create table if not exists evm_token_approval_log (
    id integer not null primary key,
    grant_id integer not null references evm_token_approval_grant(id) on delete restrict,
    client_id integer not null references program_client(id) on delete restrict,
    wallet_id integer not null references evm_wallet(id) on delete restrict,
    chain_id integer not null,              -- EIP-155 chain ID
    token_contract blob not null,           -- 20-byte ERC20 contract address
    spender_address blob not null,          -- 20-byte spender address
    value blob not null,                    -- big-endian 32-byte U256
    created_at integer not null default(unixepoch('now'))
) STRICT;

create index if not exists idx_token_approval_log_grant on evm_token_approval_log(grant_id);
create index if not exists idx_token_approval_log_client on evm_token_approval_log(client_id);
create index if not exists idx_token_approval_log_wallet on evm_token_approval_log(wallet_id);

-- Log table for unknown contract call grant usage
create table if not exists evm_unknown_call_log (
    id integer not null primary key,
    grant_id integer not null references evm_unknown_call_grant(id) on delete restrict,
    client_id integer not null references program_client(id) on delete restrict,
    wallet_id integer not null references evm_wallet(id) on delete restrict,
    chain_id integer not null,              -- EIP-155 chain ID
    contract blob not null,                 -- 20-byte target contract address
    selector blob,                          -- 4-byte function selector, null if none
    call_data blob,                         -- full call data, null if not stored
    created_at integer not null default(unixepoch('now'))
) STRICT;

create index if not exists idx_unknown_call_log_grant on evm_unknown_call_log(grant_id);
create index if not exists idx_unknown_call_log_client on evm_unknown_call_log(client_id);
create index if not exists idx_unknown_call_log_wallet on evm_unknown_call_log(wallet_id);
