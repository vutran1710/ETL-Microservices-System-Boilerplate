-- Up Migration

CREATE TABLE transactions (
    block_number BIGINT NOT NULL,
    tx_index SMALLINT NOT NULL,
    "from" VARCHAR NOT NULL,
    "to" VARCHAR NOT NULL,
    value BIGINT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    range_index BIGINT PRIMARY KEY
);
