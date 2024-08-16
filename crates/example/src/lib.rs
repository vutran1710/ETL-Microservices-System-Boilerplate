mod sink;
mod source;

use async_trait::async_trait;
use clap::Parser;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::PgConnection;
pub use sink::SinkOrderingID;
pub use source::SourceOrderingID;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

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
