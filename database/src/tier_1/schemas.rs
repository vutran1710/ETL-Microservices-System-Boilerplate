diesel::table! {
    transactions (block_number, tx_index) {
        block_number -> BigInt,
        tx_index -> SmallInt,
        from -> VarChar,
        to -> VarChar,
        value -> Numeric,
        timestamp -> Timestamp,
    }
}
