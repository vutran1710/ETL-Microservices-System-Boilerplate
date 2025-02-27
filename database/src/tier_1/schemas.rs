/*
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
*/
diesel::table! {
    actions (id) {
        id -> VarChar,
        action_type -> VarChar,
        asset_id -> BigInt,
        asset_value -> Numeric,
        usd_value -> Numeric,
        usd_price -> Numeric,
        chain_id -> BigInt,
        tx_hash -> VarChar,
        log_index -> BigInt,
        wallet_address -> VarChar,
        data -> Nullable<Jsonb>,
        block_number -> BigInt,
        block_timestamp -> BigInt,
        created_at -> Timestamp,
    }
}
