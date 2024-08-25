pub mod messages;

use async_trait::async_trait;
use database::EtlJobManager;
use database::Table;
use kanal::AsyncSender;
use messages::ChangeSet;
use messages::Message;
use std::collections::HashMap;

#[async_trait]
pub trait ETLTrait: Send + Sync {
    async fn new(source: &str, sink: &str, job_manager: &str) -> eyre::Result<Self>
    where
        Self: Sized;

    /// Return the ID of the ETL job, ID must be unique
    fn id(&self) -> String;

    /// Return EtlJobManager
    fn job_manager(&self) -> &EtlJobManager;

    /// Return the tier of the ETL, Tier must be static
    fn tier(&self) -> i64;

    /// Handle the changes and return the result
    /// Result is ChangeSet of the current tier
    async fn processing_changes(
        &self,
        changes: HashMap<Table, ChangeSet>,
    ) -> eyre::Result<HashMap<Table, ChangeSet>>;

    /// Cancel the processing related to the tables
    /// Emit cancelling message to the emitter with the tables that are cancelled OF THE CURRENT TIER
    async fn cancel_processing(&self, tables: Vec<Table>) -> eyre::Result<Vec<Table>>;

    /// Validate message tier
    /// Current Tier App only process the message sent from the lower tier
    fn validate_message_tier(&self, receiving_tier: i64) -> bool {
        receiving_tier + 1 == self.tier()
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
                self.tier(),
                msg.get_tier()
            );
            return Ok(());
        }

        let output = match msg {
            Message::DataStoreUpdated { tables, tier: _ } => {
                let changes = self.processing_changes(tables).await?;
                Message::DataStoreUpdated {
                    tables: changes,
                    tier: self.tier(),
                }
            }
            Message::CancelProcessing { tier: _, tables } => {
                let result = self.cancel_processing(tables).await?;
                Message::CancelProcessing {
                    tier: self.tier(),
                    tables: result,
                }
            }
        };

        emitter.send(output).await?;
        Ok(())
    }
}
