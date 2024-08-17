diesel::table! {
    transactions (block_tx_index) {
        block_number -> BigInt,
        tx_index -> SmallInt,
        from -> VarChar,
        to -> VarChar,
        value -> BigInt,
        timestamp -> Timestamp,
        block_tx_index -> BigInt,
    }
}
