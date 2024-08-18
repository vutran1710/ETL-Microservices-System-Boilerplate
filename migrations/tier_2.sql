CREATE TABLE buy_sell (
    "user" VARCHAR NOT NULL,
    amount BIGINT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    range_index BIGINT NOT NULL,
    PRIMARY KEY ("user", range_index)
);
