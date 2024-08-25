pub mod messages;

use async_trait::async_trait;
use database::EtlJobManager;
use database::Table;
use kanal::AsyncSender;
use messages::ChangeSet;
use messages::Message;
use std::collections::HashMap;

#[async_trait]
pub trait ETLTrait: Send + Sync + 'static {
    fn new(source: &str, sink: &str, job_manager: &str) -> eyre::Result<Self>
    where
        Self: Sized;

    /// Return the ID of the ETL job, ID must be unique
    fn id() -> String;

    /// Return EtlJobManager
    fn job_manager(&self) -> &EtlJobManager;

    /// Return the tier of the ETL, Tier must be static
    fn tier() -> i32;

    /// Handle the changes and return the result
    /// Result is ChangeSet of the current tier
    fn processing_changes(
        &self,
        changes: HashMap<Table, ChangeSet>,
    ) -> eyre::Result<HashMap<Table, ChangeSet>>;

    /// Cancel the processing related to the tables
    /// Emit cancelling message to the emitter with the tables that are cancelled OF THE CURRENT TIER
    fn cancel_processing(&self, tables: Vec<Table>) -> eyre::Result<Vec<Table>>;

    /// Validate message tier
    /// Current Tier App only process the message sent from the lower tier
    fn validate_message_tier(&self, receiving_tier: i32) -> bool {
        receiving_tier + 1 == Self::tier()
    }

    /// Process the message and send the result to the emitter
    async fn process_message(
        &self,
        msg: Message,
        emitter: AsyncSender<Message>,
    ) -> eyre::Result<()> {
        log::info!("Processing message: \n{}", msg);

        if !self.validate_message_tier(msg.get_tier()) {
            // NOTE: if by any chance the tier is not the expected one, we should log the error and return
            // instead of panicking
            log::error!(
                "Tier mismatch: expected: {}, got: {}, skipping message processing",
                Self::tier(),
                msg.get_tier()
            );
            return Ok(());
        }

        let output = match msg {
            Message::DataStoreUpdated { tables, tier: _ } => {
                let changes = self.processing_changes(tables)?;
                Message::DataStoreUpdated {
                    tables: changes,
                    tier: Self::tier(),
                }
            }
            Message::CancelProcessing { tier: _, tables } => {
                let result = self.cancel_processing(tables)?;
                Message::CancelProcessing {
                    tier: Self::tier(),
                    tables: result,
                }
            }
        };

        emitter.send(output).await?;
        Ok(())
    }
}

/*
EXAMPLE:
fn handle_data(
    table: &Table,
    changes: &ChangeSet,
    source: &mut PgConnection,
    sink: &mut PgConnection,
    state: &mut State,
    sink_changes: &mut HashMap<Table, ChangeSet>,
) -> eyre::Result<()> {
    todo!("Implement processing")
}

create_etl_job!(
    id => "job_id_abc",
    tier => 1,
    state => State,
    handle_data
);
*/
#[macro_export]
macro_rules! create_etl_job {
    (
        id => $id:expr,
        tier => $tier:expr,
        state => $state:ident,
        $processing:expr
    ) => {
        #[derive(Clone)]
        pub struct Etl {
            source: Arc<Mutex<PgConnection>>,
            sink: Arc<Mutex<PgConnection>>,
            jm: EtlJobManager,
            state: Arc<Mutex<$state>>,
        }

        #[async_trait]
        impl ETLTrait for Etl {
            fn new(source: &str, sink: &str, job_manager: &str) -> eyre::Result<Self> {
                Ok(Etl {
                    source: Arc::new(Mutex::new(create_pg_connection(source))),
                    sink: Arc::new(Mutex::new(create_pg_connection(sink))),
                    jm: EtlJobManager::initialize(job_manager, $id, $tier),
                    state: Arc::new(Mutex::new($state::default())),
                })
            }

            fn id() -> String {
                $id.to_string()
            }

            fn job_manager(&self) -> &EtlJobManager {
                &self.jm
            }

            fn tier() -> i32 {
                $tier
            }

            fn processing_changes(
                &self,
                changes: HashMap<Table, ChangeSet>,
            ) -> eyre::Result<HashMap<Table, ChangeSet>> {
                let mut sink_changes: HashMap<Table, ChangeSet> = HashMap::new();

                for (table, changes) in changes.iter() {
                    {
                        let mut source_db = self.source.lock().unwrap();
                        let source_db = source_db.deref_mut();

                        let mut sink_db = self.sink.lock().unwrap();
                        let sink_db = sink_db.deref_mut();

                        let mut state = self.state.lock().unwrap();
                        let state = state.deref_mut();

                        $processing(table, changes, source_db, sink_db, state, &mut sink_changes)?;
                    }
                }
                Ok(sink_changes)
            }

            fn cancel_processing(&self, _tables: Vec<Table>) -> eyre::Result<Vec<Table>> {
                todo!("Implement cancel-processing")
            }
        }
    };
}
