use diesel::data_types::PgNumeric;
use diesel::data_types::PgTimestamp;
use diesel::prelude::*;
mod schemas;
use crate::OrderingID;
use serde::Deserialize;
use serde::Serialize;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schemas::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub block_number: i64,
    pub tx_index: i16,
    pub from: String,
    pub to: String,
    pub value: PgNumeric,
    pub timestamp: PgTimestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueryID {
    pub block_number: i64,
    pub tx_index: i16,
}

impl OrderingID for QueryID {}
