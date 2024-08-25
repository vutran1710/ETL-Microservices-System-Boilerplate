use async_trait::async_trait;
use chrono::NaiveDate;
use common::create_etl_job;
use common::messages::ChangeSet;
use common::ETLTrait;
use database::create_pg_connection;
use database::tier_2;
use database::tier_3::BalancePerDate;
use database::EtlJobManager;
use database::PgConnection;
use database::RowStream;
use database::Table;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

type User = String;
type BalanceState = HashMap<User, Vec<(NaiveDate, i64)>>;

fn state_to_rows(state: &BalanceState) -> Vec<BalancePerDate> {
    state
        .iter()
        .map(|(k, v)| {
            v.into_iter().map(|(date, balance)| BalancePerDate {
                user: k.clone(),
                date: date.clone(),
                balance: balance.clone(),
            })
        })
        .flatten()
        .collect()
}

fn process_buy_sell(buy_sell: tier_2::BuySell, state: &mut BalanceState) -> eyre::Result<()> {
    log::info!("Processing buy-sell: {:?}", buy_sell);
    let user = buy_sell.user.clone();
    let date = buy_sell.timestamp.date();

    if state.contains_key(&user) {
        let records = state.get_mut(&user).unwrap();
        let last_record = records.last().unwrap();
        let last_date = last_record.0;
        let last_balance = last_record.1;
        if last_date == date {
            records.pop();
            records.push((date, last_balance + buy_sell.amount));
        } else {
            records.push((date, last_balance + buy_sell.amount));
        }
    } else {
        state.insert(user, vec![(date, buy_sell.amount)]);
    }

    Ok(())
}

fn handle_data(
    table: &Table,
    changes: &ChangeSet,
    source: &mut PgConnection,
    sink: &mut PgConnection,
    state: &mut BalanceState,
    sink_changes: &mut HashMap<Table, ChangeSet>,
) -> eyre::Result<()> {
    log::info!("Processing changes for table: {:?}", table);
    match table {
        Table::Tier2(tier_2::Table::BuySell) => {
            log::info!("Processing buy-sell");
            let changes = changes.lowest_ranges();
            let stream = tier_2::BuySell::query(source, &changes.ranges())?;

            for row in stream {
                log::info!("Processing row: {:?}", row);
                process_buy_sell(row, state)?;
            }

            BalancePerDate::insert_many(sink, state_to_rows(&state))?;
        }

        _ => eyre::bail!("Unsupported table: {}", table),
    }
    Ok(())
}

create_etl_job!(
    id => "job_id_2",
    tier => 2,
    state => BalanceState,
    handle_data
);
