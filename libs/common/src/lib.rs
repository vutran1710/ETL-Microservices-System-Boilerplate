use async_trait::async_trait;
use serde_json::Value;

pub mod messages;
pub mod source_sink;

#[async_trait]
pub trait ETLTrait {
    async fn new() -> eyre::Result<Self>
    where
        Self: Sized;
    async fn process_one(&self, id: Value) -> eyre::Result<()>;
    async fn process_all(&self) -> eyre::Result<()>;
    async fn process_ids(&self, ids: Vec<Value>) -> eyre::Result<()>;
    async fn process_range(&self, from: Value, to: Option<Value>) -> eyre::Result<()>;
    async fn process_message(&self, msg: messages::Message) -> eyre::Result<()> {
        match msg {
            messages::Message::ProcessAll => self.process_all().await,
            messages::Message::ProcessOne { id } => self.process_one(id).await,
            messages::Message::ProcessIds { ids } => self.process_ids(ids).await,
            messages::Message::ProcessRange { from, to } => self.process_range(from, to).await,
        }
    }
}
