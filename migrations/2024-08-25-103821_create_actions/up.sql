-- Your SQL goes here
CREATE TYPE action_types AS ENUM ('SEND', 'RECEIVE');

CREATE TABLE IF NOT EXISTS actions
(
    id              varchar(255) PRIMARY KEY NOT NULL UNIQUE,
    action_type     action_types  NOT NULL,
    asset_id        BIGINT                   NOT NULL CHECK (asset_id > 0) REFERENCES assets (id),
    asset_value     NUMERIC                  NOT NULL CHECK (asset_value >= 0),
    usd_value       NUMERIC                  NOT NULL CHECK (usd_value >= 0),
    usd_price       NUMERIC                  NOT NULL CHECK (usd_price >= 0),
    chain_id        BIGINT                   NOT NULL,
    tx_hash         VARCHAR(100)             NOT NULL,
    log_index       BIGINT                   NOT NULL,
    wallet_address  VARCHAR(80)              NOT NULL,
    data            jsonb                    NULL,
    block_number    BIGINT                   NOT NULL,
    block_timestamp BIGINT                   NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_actions_tx_hash ON actions (tx_hash);
CREATE INDEX idx_actions_wallet_address ON actions (wallet_address);
CREATE INDEX idx_actions_block_timestamp ON actions (block_timestamp);
CREATE INDEX idx_actions_chain_id_block_number ON actions (chain_id, block_number asc);
