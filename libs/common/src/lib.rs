pub mod messages;
pub mod source_sink;

use async_trait::async_trait;
use kanal::AsyncSender;
use std::collections::HashMap;

use database::interfaces::OrderingID;
use messages::ChangeSet;
use messages::Message;

#[async_trait]
pub trait ETLTrait<T: OrderingID, P: OrderingID> {
    async fn new(source: &str, sink: &str) -> eyre::Result<Self>
    where
        Self: Sized;

    /// Return the tier of the ETL, Tier must be static
    fn tier(&self) -> i64;

    /// Handle the changes and return the result
    async fn handling_changes(
        &self,
        changes: HashMap<String, ChangeSet<T>>,
    ) -> eyre::Result<Message<P>>;

    /// Process the message and send the result to the emitter
    async fn process_message(
        &self,
        msg: Message<T>,
        emitter: AsyncSender<Message<P>>,
    ) -> eyre::Result<()> {
        let current_tier = self.tier();
        match msg {
            Message::DataStoreUpdated { tier, tables } => {
                log::info!(
                    "Handling DataStoreUpdated: tier: {}, tables: {:?}",
                    tier,
                    tables
                );

                if tier != current_tier {
                    eyre::bail!("Tier mismatch: expected: {}, got: {}", current_tier, tier);
                }

                let result = self.handling_changes(tables).await?;
                emitter.send(result).await?;
                Ok(())
            }
        }
    }
}
