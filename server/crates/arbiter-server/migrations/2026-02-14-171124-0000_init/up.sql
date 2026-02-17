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