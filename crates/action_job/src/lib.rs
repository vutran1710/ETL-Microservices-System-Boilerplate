use async_trait::async_trait;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_1;
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

#[allow(dead_code)]
pub struct Etl {
    source: Arc<Mutex<PgConnection>>,
    sink: Arc<Mutex<PgConnection>>,
    jm: EtlJobManager,
}

const JOB_ID: &str = "etl-example-1";
const JOB_TIER: i32 = 1;

impl Etl {}

fn process_action(
    _pool: &mut PgConnection,
    action: tier_1::Action,
    _sink_changes: &mut HashMap<Table, ChangeSet>,
) -> eyre::Result<()> {
    log::info!("Processing action: {:?}", action);
    todo!("Implement process_action");
}

// Implement ETLTrait here -----------------------------------------------------
#[async_trait]
impl ETLTrait for Etl {
    async fn new(source: &str, sink: &str, job_manager: &str) -> eyre::Result<Self> {
        Ok(Etl {
            source: Arc::new(Mutex::new(create_pg_connection(source))),
            sink: Arc::new(Mutex::new(create_pg_connection(sink))),
            jm: EtlJobManager::initialize(job_manager, JOB_ID, JOB_TIER),
        })
    }

    fn id() -> String {
        JOB_ID.to_string()
    }

    fn job_manager(&self) -> &EtlJobManager {
        &self.jm
    }

    fn tier() -> i32 {
        JOB_TIER
    }

    async fn processing_changes(
        &self,
        changes: HashMap<Table, ChangeSet>,
    ) -> eyre::Result<HashMap<Table, ChangeSet>> {
        let mut sink_changes: HashMap<Table, ChangeSet> = HashMap::new();

        for (table, changes) in changes.iter() {
            log::info!("Processing changes for table: {:?}", table);
            match table {
                Table::Tier1(tier_1::Table::Actions) => {
                    log::info!("Processing actions...");
                    let stream = tier_1::Action::query(self.source.clone(), &changes.ranges());
                    pin_mut!(stream);
                    while let Some(row) = stream.next().await {
                        log::info!("Processing row: {:?}", row);
                        let mut pool = self.sink.lock().unwrap();
                        let pool = pool.deref_mut();
                        process_action(pool, row, &mut sink_changes)?;
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
