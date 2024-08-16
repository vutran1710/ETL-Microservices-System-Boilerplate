diesel::table! {
    buy_sell (user, block_number, tx_index) {
        user -> VarChar,
        amount -> BigInt,
        block_number -> BigInt,
        tx_index -> SmallInt,
        timestamp -> Timestamp,
    }
}
