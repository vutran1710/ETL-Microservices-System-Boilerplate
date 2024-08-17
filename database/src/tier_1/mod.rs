use diesel::data_types::PgTimestamp;
use diesel::prelude::*;
mod schemas;
use crate::OrderingID;
use crate::RowStream;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable)]
#[diesel(table_name = schemas::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub block_number: i64,
    pub tx_index: i16,
    pub from: String,
    pub to: String,
    pub value: i64,
    pub timestamp: PgTimestamp,
    pub block_tx_index: i64,
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    Transactions,
}

// QueryID is used to query the database -------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueryID {
    pub block_tx_index: i64,
}

impl OrderingID for QueryID {}

// Implement RowStream for Transaction -------------------------------------------------------
impl RowStream<QueryID> for Transaction {
    fn query_range(
        pool: &mut PgConnection,
        range: &crate::RangeID<QueryID>,
    ) -> eyre::Result<Vec<Self>> {
        use schemas::transactions::dsl::*;
        let xfrom = range.from.clone();
        let xto = range.to.clone();
        let rows = transactions
            .filter(block_number.ge(xfrom.block_tx_index))
            .filter(block_number.le(xto.block_tx_index))
            .load(pool)?;
        Ok(rows)
    }
}
