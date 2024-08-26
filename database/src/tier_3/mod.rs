mod schemas;

use crate::Range;
use crate::RangeQuery;
use crate::RowStream;
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = schemas::balance_per_date)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalancePerDate {
    pub user: String,
    pub balance: i64,
    pub date: NaiveDate,
}

impl BalancePerDate {
    pub fn insert_many(pool: &mut PgConnection, rows: Vec<Self>) -> eyre::Result<()> {
        use diesel::pg::upsert::excluded;
        use schemas::balance_per_date::dsl::*;

        let inserted: Vec<Self> = diesel::insert_into(balance_per_date)
            .values(&rows)
            .on_conflict((user, date))
            .do_update()
            .set(balance.eq(excluded(balance)))
            .get_results(pool)?;
        log::info!("Inserted {} rows", inserted.len());
        Ok(())
    }
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
#[serde(rename_all = "lowercase")]
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
    fn query_range(pool: &mut PgConnection, query: &RangeQuery) -> eyre::Result<Vec<Self>> {
        let user_filter: Filter = serde_json::from_value(query.filters.clone())?;
        if let Range::Date {
            from: from_date,
            to: _,
        } = query.range
        {
            use schemas::balance_per_date::dsl::*;
            let rows = balance_per_date
                .filter(user.eq(user_filter.user))
                .filter(date.ge(from_date))
                .order((date, date.asc()))
                .load(pool)?;
            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}
