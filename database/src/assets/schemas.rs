diesel::table! {
    assets (id) {
        id -> BigSerial,
        address -> VarChar,
        name -> VarChar,
        decimals -> Integer,
        symbol -> VarChar,
        chain_id -> BigInt,
    }
}
