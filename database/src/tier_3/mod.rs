mod schemas;

use crate::QueryWithRange;
use crate::Range;
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

#[derive(Deserialize, Serialize)]
pub struct Filter {
    pub user: String,
}

// Implement RowStream for BalancePerDate -------------------------------------------------------
impl RowStream for BalancePerDate {
    fn query_range(pool: &mut PgConnection, query: &QueryWithRange) -> eyre::Result<Vec<Self>> {
        let user_filter: Filter = serde_json::from_value(query.filters.clone())?;
        if let Range::Date {
            from: from_date,
            to: to_date,
        } = query.range
        {
            use schemas::balance_per_date::dsl::*;
            let rows = balance_per_date
                .filter(user.eq(user_filter.user))
                .filter(date.ge(from_date))
                .filter(date.le(to_date))
                .load(pool)?;
            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}
