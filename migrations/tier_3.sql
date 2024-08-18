CREATE TABLE balance_per_date (
    "user" VARCHAR NOT NULL,
    balance BIGINT NOT NULL,
    date DATE NOT NULL,
    PRIMARY KEY ("user", date)
);
