use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
mod schemas;
use crate::QueryWithRange;
use crate::Range;
use crate::RowStream;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = schemas::actions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Action {
    pub id: String,
    pub action_type: String,
    pub asset_id: i64,
    pub asset_value: BigDecimal,
    pub usd_value: BigDecimal,
    pub usd_price: BigDecimal,
    pub chain_id: i64,
    pub tx_hash: String,
    pub log_index: i64,
    pub wallet_address: String,
    pub data: Option<serde_json::Value>,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub created_at: NaiveDateTime,
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    Actions,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChainIdFilter {
    pub chain_id: i64,
}

// Implement RowStream for Transaction -------------------------------------------------------
impl RowStream for Action {
    fn query_range(pool: &mut PgConnection, query: &QueryWithRange) -> eyre::Result<Vec<Self>> {
        if let Range::Numeric {
            from: from_block_number,
            to: to_block_number,
        } = query.range
        {
            use schemas::actions::dsl::*;
            let chain_id_filter: ChainIdFilter =
                serde_json::from_value(query.filters.clone()).expect("no chain_id filter found");

            let rows = actions
                .filter(block_number.ge(from_block_number))
                .filter(block_number.le(to_block_number))
                .filter(chain_id.eq(chain_id_filter.chain_id))
                .load(pool)?;

            Ok(rows)
        } else {
            Err(eyre::eyre!("Invalid range type"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rand::Rng;

    fn create_mock_actions(conn: &mut PgConnection, count: usize) {
        use super::schemas::actions::dsl::*;

        let mut rng = rand::thread_rng();
        let mut mock_actions = Vec::with_capacity(count);

        for i in 0..count {
            let mock_block_number: i64 = (i as i64) + 5;
            let mock_tx_index = rng.gen_range(1..10);

            let action = Action {
                id: format!("tx-{}", i),
                action_type: "buy".to_string(),
                asset_id: 1,
                asset_value: BigDecimal::from(100),
                usd_value: BigDecimal::from(100),
                usd_price: BigDecimal::from(1),
                chain_id: 1,
                tx_hash: format!("tx-hash-{}", i),
                log_index: mock_tx_index,
                wallet_address: "0x1234567890".to_string(),
                data: None,
                block_number: mock_block_number,
                block_timestamp: Utc::now().timestamp(),
                created_at: Utc::now().naive_utc(),
            };
            mock_actions.push(action);
        }

        log::info!("Execute.........");
        let result: Vec<Action> = diesel::insert_into(actions)
            .values(&mock_actions)
            .get_results(conn)
            .unwrap();
        log::info!("Result: {:?}", result);
        log::info!("Mock transactions inserted");
        assert_eq!(result.len(), count);
    }

    #[test]
    fn test_create_txs() {
        env_logger::try_init().ok();
        log::info!("Connecting to database");
        let mut conn = PgConnection::establish("postgres://postgres:postgres@localhost:5432/mc2")
            .expect("Error connecting to database");

        log::info!("Creating mock actions...");
        create_mock_actions(&mut conn, 5);
    }
}
