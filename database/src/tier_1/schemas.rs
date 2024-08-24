diesel::table! {
    transactions (timestamp) {
        block_number -> BigInt,
        tx_index -> SmallInt,
        from -> VarChar,
        to -> VarChar,
        value -> BigInt,
        timestamp -> Timestamp,
    }
}
