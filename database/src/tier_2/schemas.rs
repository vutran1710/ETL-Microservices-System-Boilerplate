diesel::table! {
    buy_sell (user, range_index) {
        user -> VarChar,
        amount -> BigInt,
        timestamp -> Timestamp,
        range_index -> BigInt,
    }
}
