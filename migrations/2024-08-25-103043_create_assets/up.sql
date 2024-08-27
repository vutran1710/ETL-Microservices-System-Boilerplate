-- Your SQL goes here
-- Assets
CREATE TABLE IF NOT EXISTS assets
(
    id       SERIAL PRIMARY KEY,
    address  VARCHAR(80)  NOT NULL,
    name     VARCHAR(255) NOT NULL,
    decimals INTEGER      NOT NULL CHECK (decimals >= 0),
    symbol   VARCHAR(255) NOT NULL,
    chain_id BIGINT       NOT NULL check ( chain_id >= 0),
    UNIQUE (address, chain_id)
);

-- Index asset unique address + chain_id
CREATE UNIQUE INDEX IF NOT EXISTS idx_assets_address_chain_id ON assets (address, chain_id);
CREATE INDEX idx_assets_symbol ON assets (symbol);

-- Insert default assets
INSERT INTO assets (address, name, decimals, symbol, chain_id)
values ('0xdac17f958d2ee523a2206206994597c13d831ec7', 'USDT', 6, 'USDT', 1);
INSERT INTO assets (address, name, decimals, symbol, chain_id)
values ('0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2', 'ETH', 18, 'ETH', 1);
