use async_trait::async_trait;
use clap::Parser;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_1;
use database::tier_2;
use database::PgConnection;
use database::RowStream;
use database::Table;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

pub use database::tier_1::QueryID as SourceOrderingID;
pub use database::tier_2::QueryID as SinkOrderingID;

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

impl Etl {
    async fn process_transaction(
        &self,
        transaction: tier_1::Transaction,
        _sink_changes: &mut HashMap<String, ChangeSet<SinkOrderingID>>,
    ) -> eyre::Result<()> {
        let _sell = tier_2::BuySell {
            user: transaction.from,
            amount: transaction.value * -1,
            timestamp: transaction.timestamp,
            block_tx_index: transaction.block_tx_index,
        };
        let _buy = tier_2::BuySell {
            user: transaction.to,
            amount: transaction.value,
            timestamp: transaction.timestamp,
            block_tx_index: transaction.block_tx_index,
        };
        // Insert the sell and buy transactions into the sink
        Ok(())
    }
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
        changes: HashMap<String, ChangeSet<SourceOrderingID>>,
    ) -> eyre::Result<HashMap<String, ChangeSet<SinkOrderingID>>> {
        let mut sink_changes = HashMap::new();

        for (table_name, changes) in changes.iter() {
            let table = Table::from_str(table_name)?;
            let ranges = changes.get_change_ranges();

            match table {
                Table::Tier1(tier_1::Table::Transactions) => {
                    let stream = tier_1::Transaction::query(self.source.clone(), &ranges);
                    pin_mut!(stream);
                    while let Some(row) = stream.next().await {
                        self.process_transaction(row, &mut sink_changes).await?;
                    }
                }

                _ => eyre::bail!("Unsupported table: {}", table_name),
            }
        }
        Ok(sink_changes)
    }

    async fn cancel_processing(&self, _tables: Vec<String>) -> eyre::Result<Vec<String>> {
        Ok(vec![])
    }
}
