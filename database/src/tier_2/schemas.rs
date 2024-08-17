diesel::table! {
    buy_sell (user, block_tx_index) {
        user -> VarChar,
        amount -> BigInt,
        timestamp -> Timestamp,
        block_tx_index -> BigInt,
    }
}
