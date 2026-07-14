-- Add up migration script for owned_assets here
CREATE TABLE IF NOT EXISTS owned_assets (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    user_id BIGSERIAL NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset_id BIGSERIAL NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    bought_for DOUBLE PRECISION NOT NULL,
    quantity_owned DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, asset_id)
);