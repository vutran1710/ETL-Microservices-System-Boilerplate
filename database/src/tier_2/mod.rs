mod schemas;

use crate::QueryWithRange;
use crate::Range;
use crate::RowStream;
use diesel::data_types::PgTimestamp;
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
#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schemas::buy_sell)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BuySell {
    pub user: String,
    pub amount: i64,
    pub timestamp: PgTimestamp,
    // NOTE: derived from Tx's range_index
    pub range_index: i64,
}

impl BuySell {
    pub fn insert_many(pool: &mut PgConnection, rows: Vec<Self>) -> eyre::Result<()> {
        use schemas::buy_sell::dsl::*;

        insert_into(buy_sell).values(&rows).execute(pool)?;
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

impl From<&BuySell> for QueryWithRange {
    fn from(value: &BuySell) -> Self {
        QueryWithRange {
            range: Range::Numeric {
                from: value.range_index,
                to: value.range_index,
            },
            filters: json!({ "user": value.user }),
        }
    }
}

// Implement RowStream for BuySell -------------------------------------------------------
impl RowStream for BuySell {
    fn query_range(pool: &mut PgConnection, query: &QueryWithRange) -> eyre::Result<Vec<Self>> {
        use schemas::buy_sell::dsl::*;
        let user_filter: Filter = serde_json::from_value(query.filters.clone())?;

        if let Range::Numeric {
            from: range_from,
            to: range_to,
        } = query.range
        {
            let rows = buy_sell
                .filter(user.eq(user_filter.user))
                .filter(range_index.ge(range_from))
                .filter(range_index.le(range_to))
                .load(pool)?;

            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}
