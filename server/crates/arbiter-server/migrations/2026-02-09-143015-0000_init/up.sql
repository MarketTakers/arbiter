-- This is a singleton 
create table if not exists arbiter_settings (
    root_key_enc blob, -- if null, means wasn't bootstrapped yet
    cert_key blob not null,
    cert blob not null
) STRICT;

create table if not exists key_identity(
    id integer primary key,
    name text not null,
    public_key text not null,
    created_at integer not null default (unixepoch('now')),
    updated_at integer not null default (unixepoch('now'))
) STRICT;

create table if not exists useragent_client (
    id integer primary key,
    key_identity_id integer not null references key_identity(id) on delete cascade,
    created_at integer not null default (unixepoch('now')),
    updated_at integer not null default (unixepoch('now'))
) STRICT;


create table if not exists program_client(
    id integer primary key,
    key_identity_id integer not null references key_identity(id) on delete cascade,
    created_at integer not null default (unixepoch('now')),
    updated_at integer not null default (unixepoch('now'))
) STRICT;