-- Add migration script here
-- create table if not exists identity (
--     id integer primary key AUTOINCREMENT,
--     iden_key blob UNIQUE,
--     iden_value blob,
--     createdAt TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
-- );

create table if not exists identity (
    id integer primary key AUTOINCREMENT,
    user text,
    iden_key blob,
    iden_value blob,
    createdAt TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user, iden_key)
);
