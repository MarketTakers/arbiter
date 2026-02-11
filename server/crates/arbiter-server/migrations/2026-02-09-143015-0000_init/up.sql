create table if not exists aead_encrypted (
    id INTEGER not null PRIMARY KEY,
    current_nonce integer not null default(1), -- if re-encrypted, this should be incremented
    ciphertext blob not null,
    tag blob not null,
    schema_version integer not null default(1) -- server would need to reencrypt, because this means that we have changed algorithm
) STRICT;

-- This is a singleton
create table if not exists arbiter_settings (
    id INTEGER not null PRIMARY KEY CHECK (id = 1), -- singleton row, id must be 1
    root_key_id integer references aead_encrypted (id) on delete RESTRICT, -- if null, means wasn't bootstrapped yet
    cert_key blob not null,
    cert blob not null
) STRICT;

create table if not exists key_identity (
    id integer not null primary key,
    name text not null,
    public_key text not null,
    created_at integer not null default(unixepoch ('now')),
    updated_at integer not null default(unixepoch ('now'))
) STRICT;

create table if not exists useragent_client (
    id integer not null primary key,
    key_identity_id integer not null references key_identity (id) on delete cascade,
    created_at integer not null default(unixepoch ('now')),
    updated_at integer not null default(unixepoch ('now'))
) STRICT;

create table if not exists program_client (
    id integer not null primary key,
    key_identity_id integer not null references key_identity (id) on delete cascade,
    created_at integer not null default(unixepoch ('now')),
    updated_at integer not null default(unixepoch ('now'))
) STRICT;