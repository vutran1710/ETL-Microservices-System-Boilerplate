pub mod messages;

use async_trait::async_trait;
use kanal::AsyncSender;
use std::collections::HashMap;

use database::interfaces::OrderingID;
use messages::ChangeSet;
use messages::Message;

#[async_trait]
pub trait ETLTrait<T: OrderingID, P: OrderingID>: Send + Sync {
    async fn new(source: &str, sink: &str) -> eyre::Result<Self>
    where
        Self: Sized;

    /// Return the tier of the ETL, Tier must be static
    fn tier(&self) -> i64;

    /// Handle the changes and return the result
    /// Result is ChangeSet of the current tier
    async fn processing_changes(
        &self,
        changes: HashMap<String, ChangeSet<T>>,
    ) -> eyre::Result<HashMap<String, ChangeSet<P>>>;

    /// Cancel the processing related to the tables
    /// Emit cancelling message to the emitter with the tables that are cancelled OF THE CURRENT TIER
    async fn cancel_processing(&self, tables: Vec<String>) -> eyre::Result<Vec<String>>;

    /// Validate message tier
    /// Current Tier App only process the message sent from the lower tier
    fn validate_message_tier(&self, receiving_tier: i64) -> bool {
        receiving_tier + 1 == self.tier()
    }

    /// Process the message and send the result to the emitter
    async fn process_message(
        &self,
        msg: Message<T>,
        emitter: AsyncSender<Message<P>>,
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

        match msg {
            Message::DataStoreUpdated { tier: _, tables } => {
                emitter
                    .send(Message::DataStoreUpdated {
                        tier: self.tier(),
                        tables: self.processing_changes(tables).await?,
                    })
                    .await?;
                Ok(())
            }
            Message::CancelProcessing { tier: _, tables } => {
                emitter
                    .send(Message::CancelProcessing {
                        tier: self.tier(),
                        tables: self.cancel_processing(tables).await?,
                    })
                    .await?;
                Ok(())
            }
        }
    }
}
