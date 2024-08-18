use async_trait::async_trait;
use clap::Parser;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_1;
use database::tier_2;
use database::PgConnection;
use database::QueryWithRange;
use database::Range;
use database::RowStream;
use database::Table;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use serde_json::json;
use std::collections::HashMap;
use std::ops::DerefMut;
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

impl Etl {
    async fn process_transaction(
        &self,
        transaction: tier_1::Transaction,
        sink_changes: &mut HashMap<Table, ChangeSet>,
    ) -> eyre::Result<()> {
        let sell = tier_2::BuySell {
            user: transaction.from,
            amount: transaction.value * -1,
            timestamp: transaction.timestamp,
            block_tx_index: transaction.block_tx_index,
        };
        let q1 = QueryWithRange {
            range: Range::Numeric {
                from: sell.block_tx_index,
                to: sell.block_tx_index,
            },
            filters: json!({ "user": sell.user }),
        };
        let buy = tier_2::BuySell {
            user: transaction.to,
            amount: transaction.value,
            timestamp: transaction.timestamp,
            block_tx_index: transaction.block_tx_index,
        };
        let q2 = QueryWithRange {
            range: Range::Numeric {
                from: buy.block_tx_index,
                to: buy.block_tx_index,
            },
            filters: json!({ "user": buy.user }),
        };

        let mut pool = self.sink.lock().unwrap();
        let pool = pool.deref_mut();
        tier_2::BuySell::insert_many(pool, vec![sell, buy])?;

        sink_changes
            .entry(Table::Tier2(tier_2::Table::BuySell))
            .or_insert(ChangeSet::new())
            .push(q1);

        sink_changes
            .entry(Table::Tier2(tier_2::Table::BuySell))
            .or_insert(ChangeSet::new())
            .push(q2);

        Ok(())
    }
}

#[async_trait]
impl ETLTrait for Etl {
    async fn new(source: &str, sink: &str) -> eyre::Result<Self> {
        Ok(Etl {
            source: Arc::new(Mutex::new(create_pg_connection(source))),
            sink: Arc::new(Mutex::new(create_pg_connection(sink))),
        })
    }

    fn tier(&self) -> i64 {
        0
    }

    async fn processing_changes(
        &self,
        changes: HashMap<Table, ChangeSet>,
    ) -> eyre::Result<HashMap<Table, ChangeSet>> {
        let mut sink_changes: HashMap<Table, ChangeSet> = HashMap::new();

        for (table, changes) in changes.iter() {
            match table {
                Table::Tier1(tier_1::Table::Transactions) => {
                    let stream = tier_1::Transaction::query(self.source.clone(), &changes.ranges());
                    pin_mut!(stream);
                    while let Some(row) = stream.next().await {
                        self.process_transaction(row, &mut sink_changes).await?;
                    }
                }

                _ => eyre::bail!("Unsupported table: {}", table),
            }
        }
        Ok(sink_changes)
    }

    async fn cancel_processing(&self, _tables: Vec<Table>) -> eyre::Result<Vec<Table>> {
        Ok(vec![])
    }
}
