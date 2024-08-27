mod schemas;

use crate::Range;
use crate::RangeQuery;
use crate::RowStream;
use chrono::DateTime;
use chrono::NaiveDateTime;

use diesel::insert_into;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use strum::Display;
use strum::EnumString;

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Display)]
pub enum Action {
    #[default]
    #[strum(serialize = "BUY")]
    Buy,
    #[strum(serialize = "SELL")]
    Sell,
}

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = schemas::buy_sell)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BuySell {
    pub user: String,
    pub amount: i64,
    pub timestamp: NaiveDateTime,
}

impl BuySell {
    pub fn insert_many(pool: &mut PgConnection, rows: Vec<Self>) -> eyre::Result<()> {
        use diesel::pg::upsert::excluded;
        use schemas::buy_sell::dsl::*;

        let inserted: Vec<Self> = insert_into(buy_sell)
            .values(&rows)
            .on_conflict((user, timestamp))
            .do_update()
            .set(amount.eq(excluded(amount)))
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
    BuySell,
}

#[derive(Deserialize, Serialize)]
pub struct Filter {
    pub user: String,
}

impl From<&BuySell> for RangeQuery {
    fn from(value: &BuySell) -> Self {
        RangeQuery {
            range: Range::Numeric {
                from: value.timestamp.and_utc().timestamp(),
                to: value.timestamp.and_utc().timestamp(),
            },
            filters: json!({ "user": value.user }),
        }
    }
}

// Implement RowStream for BuySell -------------------------------------------------------
impl RowStream for BuySell {
    fn query(pool: &mut PgConnection, query: &RangeQuery) -> eyre::Result<Vec<Self>> {
        use schemas::buy_sell::dsl::*;
        let user_filter: Filter = serde_json::from_value(query.filters.clone())?;

        if let Range::Numeric { from, to: _ } = query.range {
            let from_timestamp = DateTime::from_timestamp(from, 0).unwrap().naive_utc();
            let rows = buy_sell
                .filter(user.eq(user_filter.user))
                .filter(timestamp.ge(from_timestamp))
                .order(timestamp.asc())
                .load(pool)?;

            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}
