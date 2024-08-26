use async_trait::async_trait;
use common::create_etl_job;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_1;
use database::EtlJobManager;
use database::PgConnection;
use database::RowStream;
use database::Table;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Default)]
struct ExampleState {
    array: Vec<i32>,
}

fn handle_data(
    table: &Table,
    changes: &ChangeSet,
    source: &mut PgConnection,
    sink: &mut PgConnection,
    state: &mut ExampleState,
    sink_changes: &mut HashMap<Table, ChangeSet>,
) -> eyre::Result<()> {
    log::info!("Processing changes for table: {:?}", table);
    match table {
        Table::Tier1(tier_1::Table::Actions) => {
            log::info!("Processing actions...");
            let stream = tier_1::Action::query(source, &changes.ranges())?;
            for row in stream {
                log::info!("Processing row: {:?}", row);
            }
        }

        _ => eyre::bail!("Unsupported table: {}", table),
    }
    Ok(())
}

create_etl_job!(
    id => "job_id_abc",
    tier => 1,
    state => ExampleState,
    handle_data
);
