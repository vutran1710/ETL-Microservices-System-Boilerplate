use std::collections::HashMap;

use async_trait::async_trait;
use clap::Parser;
use common::messages::ChangeSet;
use common::messages::Message;
use common::ETLTrait;
use database::interfaces::OrderingID;
use serde::Deserialize;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source_db: String,
}

pub struct Etl {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceOrderingID(i64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SinkOrderingID((i64, i32));

impl OrderingID for SourceOrderingID {}
impl OrderingID for SinkOrderingID {}

#[async_trait]
impl ETLTrait<SourceOrderingID, SinkOrderingID> for Etl {
    async fn new(_source: &str, _sink: &str) -> eyre::Result<Self> {
        Ok(Etl {})
    }

    fn tier(&self) -> i64 {
        -1
    }

    async fn handling_changes(
        &self,
        _changes: HashMap<String, ChangeSet<SourceOrderingID>>,
    ) -> eyre::Result<Message<SinkOrderingID>> {
        Ok(Message::DataStoreUpdated {
            tier: -1,
            tables: HashMap::new(),
        })
    }
}
