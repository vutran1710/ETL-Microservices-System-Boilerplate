use common::create_etl_job;
use database::*;

#[derive(Debug, Default)]
struct ExampleState {
    #[allow(dead_code)]
    array: Vec<i32>,
}

fn handle_data(
    table: Table,
    range: RangeQuery,
    source: &mut PgConnection,
    _sink: &mut PgConnection,
    _state: &mut ExampleState,
) -> eyre::Result<Option<(Table, RangeQuery)>> {
    log::info!("Processing changes for table: {:?}", table);
    match table {
        Table::Tier1(tier_1::Table::Actions) => {
            log::info!("Processing actions...");
            let stream = tier_1::Action::query(source, &range)?;
            for row in stream {
                log::info!("Processing row: {:?}", row);
            }
        }

        _ => {
            // NOTE: dont throw, just ignore it
            return Ok(None);
        }
    }

    Ok(Some((
        Table::Tier2(tier_2::Table::BuySell),
        RangeQuery::default(),
    )))
}

create_etl_job!(
    id => "job_id_abc",
    state => ExampleState,
    handle_data
);
