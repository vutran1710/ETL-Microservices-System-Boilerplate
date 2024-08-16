use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use clap::Parser;
use common::messages::ChangeSet;

use common::ETLTrait;
use database::create_pg_connection;
use database::OrderingID;
use database::PgConnection;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source_db: String,
}

#[allow(dead_code)]
pub struct Etl {
    source: Arc<Mutex<PgConnection>>,
    sink: Arc<Mutex<PgConnection>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceOrderingID(i64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SinkOrderingID((i64, i32));

impl OrderingID for SourceOrderingID {}
impl OrderingID for SinkOrderingID {}

#[async_trait]
impl ETLTrait<SourceOrderingID, SinkOrderingID> for Etl {
    async fn new(source: &str, sink: &str) -> eyre::Result<Self> {
        Ok(Etl {
            source: Arc::new(Mutex::new(create_pg_connection(source))),
            sink: Arc::new(Mutex::new(create_pg_connection(sink))),
        })
    }

    fn tier(&self) -> i64 {
        -1
    }

    async fn processing_changes(
        &self,
        _changes: HashMap<String, ChangeSet<SourceOrderingID>>,
    ) -> eyre::Result<HashMap<String, ChangeSet<SinkOrderingID>>> {
        Ok(HashMap::new())
    }

    async fn cancel_processing(&self, _tables: Vec<String>) -> eyre::Result<Vec<String>> {
        Ok(vec![])
    }
}
