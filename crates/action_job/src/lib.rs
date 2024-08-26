use async_trait::async_trait;
use common::create_etl_job;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_1;
use database::tier_2;
use database::EtlJobManager;
use database::PgConnection;
use database::RangeQuery;
use database::RowStream;
use database::Table;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Default)]
struct ExampleState {
    array: Vec<i32>,
}

fn handle_data(
    table: Table,
    range: RangeQuery,
    source: &mut PgConnection,
    sink: &mut PgConnection,
    state: &mut ExampleState,
) -> eyre::Result<(Table, RangeQuery)> {
    log::info!("Processing changes for table: {:?}", table);
    let initial_range = range.clone_as_start();
    match table {
        Table::Tier1(tier_1::Table::Actions) => {
            log::info!("Processing actions...");
            let stream = tier_1::Action::query_range(source, &range)?;
            for row in stream {
                log::info!("Processing row: {:?}", row);
            }
        }

        _ => eyre::bail!("Unsupported table: {}", table),
    }
    Ok((Table::Tier2(tier_2::Table::BuySell), initial_range))
}

create_etl_job!(
    id => "job_id_abc",
    state => ExampleState,
    handle_data
);
