diesel::table! {
    balance_per_date (user, date) {
        user -> VarChar,
        balance -> Numeric,
        date -> Date,
    }
}
