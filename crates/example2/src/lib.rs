use async_trait::async_trait;
use chrono::NaiveDate;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_2;
use database::tier_3::BalancePerDate;
use database::EtlJobManager;
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
type BalanceState = HashMap<User, Vec<(NaiveDate, i64)>>;

fn state_to_rows(state: &BalanceState) -> Vec<BalancePerDate> {
    state
        .iter()
        .map(|(k, v)| {
            v.into_iter().map(|(date, balance)| BalancePerDate {
                user: k.clone(),
                date: date.clone(),
                balance: balance.clone(),
            })
        })
        .flatten()
        .collect()
}

#[allow(dead_code)]
pub struct Etl {
    source: Arc<Mutex<PgConnection>>,
    sink: Arc<Mutex<PgConnection>>,
    jm: EtlJobManager,
}

const JOB_ID: &str = "etl-example-2";

impl Etl {}

fn process_buy_sell(buy_sell: tier_2::BuySell, state: &mut BalanceState) -> eyre::Result<()> {
    log::info!("Processing buy-sell: {:?}", buy_sell);
    let user = buy_sell.user.clone();
    let date = buy_sell.timestamp.date();

    if state.contains_key(&user) {
        let records = state.get_mut(&user).unwrap();
        let last_record = records.last().unwrap();
        let last_date = last_record.0;
        let last_balance = last_record.1;
        if last_date == date {
            records.pop();
            records.push((date, last_balance + buy_sell.amount));
        } else {
            records.push((date, last_balance + buy_sell.amount));
        }
    } else {
        state.insert(user, vec![(date, buy_sell.amount)]);
    }

    Ok(())
}

// Implement ETLTrait here -----------------------------------------------------
#[async_trait]
impl ETLTrait for Etl {
    async fn new(source: &str, sink: &str, job_manager: &str) -> eyre::Result<Self> {
        Ok(Etl {
            source: Arc::new(Mutex::new(create_pg_connection(source))),
            sink: Arc::new(Mutex::new(create_pg_connection(sink))),
            jm: EtlJobManager::initialize(JOB_ID, job_manager),
        })
    }

    fn id() -> String {
        JOB_ID.to_string()
    }

    fn job_manager(&self) -> &EtlJobManager {
        &self.jm
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
                    let changes = changes.lowest_ranges();
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
