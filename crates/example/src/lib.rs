use async_trait::async_trait;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_1;
use database::tier_2;
use database::EtlJobManager;
use database::PgConnection;
use database::QueryWithRange;
use database::RowStream;
use database::Table;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

#[allow(dead_code)]
pub struct Etl {
    source: Arc<Mutex<PgConnection>>,
    sink: Arc<Mutex<PgConnection>>,
    jm: EtlJobManager,
}

const JOB_ID: &str = "etl-example-1";

impl Etl {}

fn process_transaction(
    pool: &mut PgConnection,
    transaction: tier_1::Transaction,
    sink_changes: &mut HashMap<Table, ChangeSet>,
) -> eyre::Result<()> {
    log::info!("Processing transaction: {:?}", transaction);
    let sell = tier_2::BuySell {
        user: transaction.from,
        amount: transaction.value * -1,
        timestamp: transaction.timestamp,
    };
    let buy = tier_2::BuySell {
        user: transaction.to,
        amount: transaction.value,
        timestamp: transaction.timestamp,
    };

    let q1 = QueryWithRange::from(&sell);
    let q2 = QueryWithRange::from(&buy);

    tier_2::BuySell::insert_many(pool, vec![sell, buy])?;

    sink_changes
        .entry(Table::Tier2(tier_2::Table::BuySell))
        .or_insert(ChangeSet::new())
        .push(q1);

    sink_changes
        .entry(Table::Tier2(tier_2::Table::BuySell))
        .or_insert(ChangeSet::new())
        .push(q2);

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

    fn id(&self) -> String {
        JOB_ID.to_string()
    }

    fn job_manager(&self) -> &EtlJobManager {
        &self.jm
    }

    fn tier(&self) -> i64 {
        1
    }

    async fn processing_changes(
        &self,
        changes: HashMap<Table, ChangeSet>,
    ) -> eyre::Result<HashMap<Table, ChangeSet>> {
        let mut sink_changes: HashMap<Table, ChangeSet> = HashMap::new();

        for (table, changes) in changes.iter() {
            log::info!("Processing changes for table: {:?}", table);
            match table {
                Table::Tier1(tier_1::Table::Transactions) => {
                    log::info!("Processing transactions");
                    let stream = tier_1::Transaction::query(self.source.clone(), &changes.ranges());
                    pin_mut!(stream);
                    while let Some(row) = stream.next().await {
                        log::info!("Processing row: {:?}", row);
                        let mut pool = self.sink.lock().unwrap();
                        let pool = pool.deref_mut();
                        process_transaction(pool, row, &mut sink_changes)?;
                    }
                }

                _ => eyre::bail!("Unsupported table: {}", table),
            }
        }
        Ok(sink_changes)
    }

    async fn cancel_processing(&self, _tables: Vec<Table>) -> eyre::Result<Vec<Table>> {
        todo!("Implement cancel-processing")
    }
}
