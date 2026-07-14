ALTER TABLE assets
ADD COLUMN symbol TEXT;

UPDATE assets
SET symbol = UPPER(REGEXP_REPLACE(name, '[^A-Za-z0-9]', '', 'g'))
WHERE symbol IS NULL;

ALTER TABLE assets
ALTER COLUMN symbol SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS assets_symbol_key ON assets(symbol);