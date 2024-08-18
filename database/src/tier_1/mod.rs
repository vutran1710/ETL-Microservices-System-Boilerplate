use diesel::data_types::PgTimestamp;
use diesel::prelude::*;
mod schemas;
use crate::QueryWithRange;
use crate::Range;
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
    // NOTE: range_index = block_number * 1000 + tx_index
    pub range_index: i64,
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    Transactions,
}

// Implement RowStream for Transaction -------------------------------------------------------
impl RowStream for Transaction {
    fn query_range(pool: &mut PgConnection, query: &QueryWithRange) -> eyre::Result<Vec<Self>> {
        if let Range::Numeric {
            from: range_from,
            to: range_to,
        } = query.range
        {
            use schemas::transactions::dsl::*;
            let rows = transactions
                .filter(range_index.ge(range_from))
                .filter(range_index.le(range_to))
                .load(pool)?;

            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}
