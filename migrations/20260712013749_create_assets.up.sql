-- Add up migration script here
CREATE TABLE IF NOT EXISTS assets (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    unit_value DOUBLE PRECISION NOT NULL
);

-- migrate create assets
-- cargo sqlx migrate info
-- cargo sqlx migrate run
-- cargo sqlx migrate revert
-- cargo sqlx migrate --help