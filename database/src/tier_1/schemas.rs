diesel::table! {
    transactions (range_index) {
        block_number -> BigInt,
        tx_index -> SmallInt,
        from -> VarChar,
        to -> VarChar,
        value -> BigInt,
        timestamp -> Timestamp,
        range_index -> BigInt,
    }
}
