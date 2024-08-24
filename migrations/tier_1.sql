-- Up Migration

CREATE TABLE transactions (
    block_number BIGINT NOT NULL,
    tx_index SMALLINT NOT NULL,
    "from" VARCHAR NOT NULL,
    "to" VARCHAR NOT NULL,
    value BIGINT NOT NULL,
    timestamp TIMESTAMP NOT NULL
);

CREATE INDEX transactions_block_number_idx ON transactions (block_number);
CREATE INDEX transactions_tx_index_idx ON transactions (tx_index);
CREATE INDEX transactions_from_idx ON transactions ("from");
CREATE INDEX transactions_to_idx ON transactions ("to");
CREATE INDEX transactions_timestamp_idx ON transactions (timestamp);
