mod schemas;

use crate::OrderingID;
use crate::RowStream;
use chrono::NaiveDate;
use diesel::data_types::PgNumeric;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable)]
#[diesel(table_name = schemas::balance_per_date)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalancePerDate {
    pub user: String,
    pub balance: PgNumeric,
    pub date: NaiveDate,
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    BalancePerDate,
}

// QueryID is used to query the database -------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueryID {
    pub user: String,
    pub date: NaiveDate,
}

impl OrderingID for QueryID {}

// Implement RowStream for BalancePerDate -------------------------------------------------------
impl RowStream<QueryID> for BalancePerDate {
    fn query_range(
        pool: &mut PgConnection,
        range: &crate::RangeID<QueryID>,
    ) -> eyre::Result<Vec<Self>> {
        use schemas::balance_per_date::dsl::*;
        let from = range.from.clone();
        let to = range.to.clone();
        let rows = balance_per_date
            .filter(user.eq(from.user))
            .filter(date.ge(from.date))
            .filter(date.le(to.date))
            .load(pool)?;
        Ok(rows)
    }
}
