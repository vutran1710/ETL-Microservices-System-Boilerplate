use async_trait::async_trait;
use chrono::NaiveDate;
use chrono::Utc;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_2;
use database::tier_2::BuySell;
use database::tier_3::BalancePerDate;
use database::PgConnection;
use database::RowStream;
use database::Table;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

type User = String;
type StateKey = (User, NaiveDate);
type BalanceState = HashMap<StateKey, i64>;

fn to_state_key(value: &BuySell) -> (User, NaiveDate) {
    let datetime = chrono::DateTime::<Utc>::from_timestamp(value.timestamp.0 / 1000, 0).unwrap();
    let datetime = datetime.naive_utc();
    (value.user.clone(), datetime.date())
}

fn state_to_rows(state: &BalanceState) -> Vec<BalancePerDate> {
    state
        .iter()
        .map(|(k, v)| BalancePerDate {
            user: k.0.clone(),
            date: k.1,
            balance: *v,
        })
        .collect()
}

#[allow(dead_code)]
pub struct Etl {
    source: Arc<Mutex<PgConnection>>,
    sink: Arc<Mutex<PgConnection>>,
}

impl Etl {}

fn process_buy_sell(buy_sell: tier_2::BuySell, state: &mut BalanceState) -> eyre::Result<()> {
    log::info!("Processing buy-sell: {:?}", buy_sell);
    let key = to_state_key(&buy_sell);

    if let Some(balance) = state.get_mut(&key) {
        *balance += buy_sell.amount;
    } else {
        state.insert(key, buy_sell.amount);
    }

    Ok(())
}

// Implement ETLTrait here -----------------------------------------------------
#[async_trait]
impl ETLTrait for Etl {
    async fn new(source: &str, sink: &str) -> eyre::Result<Self> {
        Ok(Etl {
            source: Arc::new(Mutex::new(create_pg_connection(source))),
            sink: Arc::new(Mutex::new(create_pg_connection(sink))),
        })
    }

    fn tier(&self) -> i64 {
        2
    }

    async fn processing_changes(
        &self,
        changes: HashMap<Table, ChangeSet>,
    ) -> eyre::Result<HashMap<Table, ChangeSet>> {
        let mut state: BalanceState = BalanceState::new();

        for (table, changes) in changes.iter() {
            log::info!("Processing changes for table: {:?}", table);
            match table {
                Table::Tier2(tier_2::Table::BuySell) => {
                    log::info!("Processing buy-sell");
                    let stream = tier_2::BuySell::query(self.source.clone(), &changes.ranges());
                    pin_mut!(stream);

                    while let Some(row) = stream.next().await {
                        log::info!("Processing row: {:?}", row);
                        process_buy_sell(row, &mut state)?;
                    }

                    let mut pool = self.sink.lock().unwrap();
                    let pool = pool.deref_mut();
                    BalancePerDate::insert_many(pool, state_to_rows(&state))?;
                }

                _ => eyre::bail!("Unsupported table: {}", table),
            }
        }
        Ok(HashMap::new())
    }

    async fn cancel_processing(&self, _tables: Vec<Table>) -> eyre::Result<Vec<Table>> {
        todo!("Implement cancel-processing")
    }
}
