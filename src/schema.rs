// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "action_types"))]
    pub struct ActionTypes;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ActionTypes;

    actions (id) {
        #[max_length = 255]
        id -> Varchar,
        action_type -> ActionTypes,
        asset_id -> Int8,
        asset_value -> Numeric,
        usd_value -> Numeric,
        usd_price -> Numeric,
        chain_id -> Int8,
        #[max_length = 100]
        tx_hash -> Varchar,
        log_index -> Int8,
        #[max_length = 80]
        wallet_address -> Varchar,
        data -> Nullable<Jsonb>,
        block_number -> Int8,
        block_timestamp -> Int8,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    assets (id) {
        id -> Int4,
        #[max_length = 80]
        address -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        decimals -> Int4,
        #[max_length = 255]
        symbol -> Varchar,
        chain_id -> Int8,
    }
}

diesel::joinable!(actions -> assets (asset_id));

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    assets,
);
