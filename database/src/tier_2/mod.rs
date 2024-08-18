mod schemas;

use crate::QueryWithRange;
use crate::Range;
use crate::RowStream;
use diesel::data_types::PgTimestamp;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
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
#[derive(Queryable, Selectable)]
#[diesel(table_name = schemas::buy_sell)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BuySell {
    pub user: String,
    pub amount: i64,
    pub timestamp: PgTimestamp,
    pub block_tx_index: i64,
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    BuySell,
}

#[derive(Deserialize, Serialize)]
pub struct Filter {
    pub user: String,
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
                .filter(block_tx_index.ge(range_from))
                .filter(block_tx_index.le(range_to))
                .load(pool)?;

            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}
