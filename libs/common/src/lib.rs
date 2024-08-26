pub mod messages;

use async_trait::async_trait;
use database::EtlJobManager;
use database::RangeQuery;
use database::Table;
use kanal::AsyncSender;
use messages::Message;

#[async_trait]
pub trait ETLTrait: Send + Sync + 'static {
    fn new(source: &str, sink: &str, job_manager: &str) -> eyre::Result<Self>
    where
        Self: Sized;

    /// Return the ID of the ETL job, ID must be unique
    fn id() -> String;

    /// Return EtlJobManager
    fn job_manager(&self) -> &EtlJobManager;

    /// Handle the changes and return the result
    /// Result is ChangeSet of the current tier
    fn processing_changes(
        &self,
        table: Table,
        range: RangeQuery,
    ) -> eyre::Result<(Table, RangeQuery)>;

    /// Process the message and send the result to the emitter
    async fn process_message(
        &self,
        msg: Message,
        emitter: AsyncSender<Message>,
    ) -> eyre::Result<()> {
        log::info!("Processing message: \n{}", msg);

        let output = match msg {
            Message::DataStoreUpdated { table, range } => {
                let (updated_table, updated_range) = self.processing_changes(table, range)?;
                Message::DataStoreUpdated {
                    table: updated_table,
                    range: updated_range,
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
    range: &RangeQuery,
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
                    jm: EtlJobManager::initialize(job_manager, $id),
                    state: Arc::new(Mutex::new($state::default())),
                })
            }

            fn id() -> String {
                $id.to_string()
            }

            fn job_manager(&self) -> &EtlJobManager {
                &self.jm
            }

            fn processing_changes(
                &self,
                table: Table,
                range: database::RangeQuery,
            ) -> eyre::Result<(Table, database::RangeQuery)> {
                let mut source_db = self.source.lock().unwrap();
                let source_db = source_db.deref_mut();

                let mut sink_db = self.sink.lock().unwrap();
                let sink_db = sink_db.deref_mut();

                let mut state = self.state.lock().unwrap();
                let state = state.deref_mut();

                let (updated_table, updated_query) =
                    $processing(table, range, source_db, sink_db, state)?;
                Ok((updated_table, updated_query))
            }
        }
    };
}
