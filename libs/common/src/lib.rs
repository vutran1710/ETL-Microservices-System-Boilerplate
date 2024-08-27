mod elt_job_manager;
pub mod messages;

use async_trait::async_trait;
use database::RangeQuery;
use database::Table;
pub use elt_job_manager::EtlJobManager;
use kanal::AsyncSender;
use messages::Message;

#[async_trait]
pub trait ETLTrait: Send + Sync + 'static {
    fn new(
        source: &str,
        sink: &str,
        job_manager: &str,
        emitter: AsyncSender<Message>,
    ) -> eyre::Result<Self>
    where
        Self: Sized;

    /// Return the ID of the ETL job, ID must be unique
    fn id() -> String;

    /// Return EtlJobManager
    fn job_manager(&self) -> &EtlJobManager;

    /// Return result emitter
    fn emitter(&self) -> AsyncSender<Message>;

    /// Resume the ETL job
    async fn resume(&self) -> eyre::Result<()> {
        let unfinished_jobs = self.job_manager().unfinished_jobs()?;
        let mut fut = vec![];

        for job in unfinished_jobs {
            let job_id = job.id;
            let active_request = job.active_request;
            let received_at = job.received_at;

            log::info!(
                "Resuming job with id: {}, received at: {}, active request: {}",
                job_id,
                received_at,
                active_request
            );

            let msg: Message = serde_json::from_value(active_request)?;
            let task = self.process_message(msg, job_id);
            fut.push(task);
        }

        futures::future::join_all(fut).await;

        Ok(())
    }

    /// Handle the changes and return the result
    /// Result is ChangeSet of the current tier
    fn processing_changes(
        &self,
        table: Table,
        range: RangeQuery,
    ) -> eyre::Result<Option<(Table, RangeQuery)>>;

    /// Process the message from a saved etl-job and send the result to the emitter if possible
    async fn process_message(&self, msg: Message, job_pk: i64) -> eyre::Result<()> {
        log::info!("Processing message: \n{}", msg);
        let emitter = self.emitter();

        match msg {
            Message::DataStoreUpdated { table, range } => {
                if let Some((updated_table, updated_range)) =
                    self.processing_changes(table, range)?
                {
                    let result = Message::DataStoreUpdated {
                        table: updated_table,
                        range: updated_range,
                    };
                    emitter.send(result).await?;
                } else {
                    log::info!("No output");
                }
            }
        };

        self.job_manager().mark_job_as_completed(job_pk)?;
        log::info!("Job with id: {} completed", job_pk);
        Ok(())
    }

    /// Process the message from the message queue
    async fn process_message_from_mq(&self, msg: Message) -> eyre::Result<()> {
        log::info!("Received message: \n{}", msg);
        let etl_job = self.job_manager().save(&msg)?;
        self.process_message(msg, etl_job.id).await?;
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
        use async_trait::async_trait;
        use common::messages::Message;
        use common::ETLTrait;
        use common::EtlJobManager;
        use database::create_pg_connection;
        use kanal::AsyncSender;
        use std::ops::DerefMut;
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        pub struct Etl {
            source: Arc<Mutex<PgConnection>>,
            sink: Arc<Mutex<PgConnection>>,
            jm: EtlJobManager,
            emitter: AsyncSender<Message>,
            state: Arc<Mutex<$state>>,
        }

        #[async_trait]
        impl ETLTrait for Etl {
            fn new(
                source: &str,
                sink: &str,
                job_manager: &str,
                emitter: AsyncSender<Message>,
            ) -> eyre::Result<Self> {
                Ok(Etl {
                    source: Arc::new(Mutex::new(create_pg_connection(source))),
                    sink: Arc::new(Mutex::new(create_pg_connection(sink))),
                    jm: EtlJobManager::new(job_manager, $id),
                    emitter,
                    state: Arc::new(Mutex::new($state::default())),
                })
            }

            fn id() -> String {
                $id.to_string()
            }

            fn job_manager(&self) -> &EtlJobManager {
                &self.jm
            }

            fn emitter(&self) -> AsyncSender<Message> {
                self.emitter.clone()
            }

            fn processing_changes(
                &self,
                table: Table,
                range: database::RangeQuery,
            ) -> eyre::Result<Option<(Table, database::RangeQuery)>> {
                let mut source_db = self.source.lock().unwrap();
                let source_db = source_db.deref_mut();

                let mut sink_db = self.sink.lock().unwrap();
                let sink_db = sink_db.deref_mut();

                let mut state = self.state.lock().unwrap();
                let state = state.deref_mut();

                if let Some((updated_table, updated_query)) =
                    $processing(table, range, source_db, sink_db, state)?
                {
                    Ok(Some((updated_table, updated_query)))
                } else {
                    Ok(None)
                }
            }
        }
    };
}
