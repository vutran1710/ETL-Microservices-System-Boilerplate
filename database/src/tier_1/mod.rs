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
#[diesel(table_name = schemas::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub block_number: i64,
    pub tx_index: i16,
    pub from: String,
    pub to: String,
    pub value: i64,
    pub timestamp: NaiveDateTime,
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Table {
    #[strum(ascii_case_insensitive)]
    #[default]
    Transactions,
}

// Implement RowStream for Transaction -------------------------------------------------------
impl RowStream for Transaction {
    fn query_range(pool: &mut PgConnection, query: &QueryWithRange) -> eyre::Result<Vec<Self>> {
        if let Range::Numeric {
            from: from_timestamp,
            to: to_timestamp,
        } = query.range
        {
            use schemas::transactions::dsl::*;
            let rows = transactions
                .filter(timestamp.ge(from_timestamp))
                .filter(timestamp.le(to_timestamp))
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

    fn create_mock_transactions(conn: &mut PgConnection, count: usize) {
        use super::schemas::transactions::dsl::*;

        let mut rng = rand::thread_rng();
        let mut mock_transactions = Vec::with_capacity(count);
        let users = vec!["Alice", "Bob", "David"];

        for i in 0..count {
            let mock_block_number: i64 = (i as i64) + 5;
            let mock_tx_index = rng.gen_range(1..10);
            let mock_range_index = mock_block_number * 1000 + mock_tx_index;

            let transaction = Transaction {
                block_number: mock_block_number,
                tx_index: mock_tx_index as i16,
                from: users[i % users.len()].to_string(),
                to: users[(i + 1) % users.len()].to_string(),
                value: rng.gen_range(1..10),
                timestamp: Utc::now().naive_utc() - chrono::Duration::days((50 - i) as i64),
            };
            mock_transactions.push(transaction);
        }

        log::info!("Execute.........");
        let result: Vec<Transaction> = diesel::insert_into(transactions)
            .values(&mock_transactions)
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
        let mut conn =
            PgConnection::establish("postgres://postgres:postgres@localhost:5432/postgres")
                .expect("Error connecting to database");

        log::info!("Creating mock transactions");
        create_mock_transactions(&mut conn, 5);
    }
}
