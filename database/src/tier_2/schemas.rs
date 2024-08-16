diesel::table! {
    buy_sell (user, block_number, tx_index) {
        user -> VarChar,
        action -> VarChar,
        amount -> Numeric,
        block_number -> BigInt,
        tx_index -> SmallInt,
    }
}
