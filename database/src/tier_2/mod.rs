mod schemas;

use crate::OrderingID;
use crate::RowStream;
use diesel::data_types::PgNumeric;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

// Database tables are defined here ------------------------------------------------------
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

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    BuySell,
}

// QueryID is used to query the database -------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueryID {
    pub user: String,
    pub block_number: i64,
    pub tx_index: i16,
}

impl OrderingID for QueryID {}

// Implement RowStream for BuySell -------------------------------------------------------
impl RowStream<QueryID> for BuySell {
    fn query_range(
        pool: &mut PgConnection,
        range: &crate::RangeID<QueryID>,
    ) -> eyre::Result<Vec<Self>> {
        use schemas::buy_sell::dsl::*;
        let from = range.from.clone();
        let to = range.to.clone();
        let rows = buy_sell
            .filter(user.eq(from.user))
            .filter(block_number.ge(from.block_number))
            .filter(block_number.le(to.block_number))
            .filter(tx_index.ge(from.tx_index))
            .filter(tx_index.le(to.tx_index))
            .load(pool)?;
        Ok(rows)
    }
}
