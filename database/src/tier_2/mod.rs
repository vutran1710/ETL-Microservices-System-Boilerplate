mod schemas;

use crate::OrderingID;
use diesel::data_types::PgNumeric;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schemas::buy_sell)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BuySell {
    pub user: String,
    pub action: String,
    pub amount: PgNumeric,
    pub block_number: i64,
    pub tx_index: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueryID {
    pub user: String,
    pub block_number: i64,
    pub tx_index: i16,
}

impl OrderingID for QueryID {}
