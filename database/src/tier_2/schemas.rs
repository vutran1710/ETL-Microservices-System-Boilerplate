diesel::table! {
    buy_sell (user, timestamp) {
        user -> VarChar,
        amount -> BigInt,
        timestamp -> Timestamp,
    }
}
