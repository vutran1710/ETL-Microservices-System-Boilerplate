CREATE TABLE buy_sell (
    "user" VARCHAR NOT NULL,
    amount BIGINT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    PRIMARY KEY ("user", timestamp)
);

CREATE INDEX buy_sell_user_idx ON buy_sell ("user");
CREATE INDEX buy_sell_timestamp_idx ON buy_sell (timestamp);
