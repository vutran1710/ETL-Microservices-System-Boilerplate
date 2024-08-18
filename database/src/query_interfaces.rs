use async_stream::stream;
use chrono::NaiveDate;
use diesel::PgConnection;
use futures_core::stream::Stream;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

/// Range is [from, to]: both are inclusive
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Range {
    Numeric { from: i64, to: i64 },
    Date { from: NaiveDate, to: NaiveDate },
}

impl Range {
    fn validate(&self) -> bool {
        match self {
            Range::Numeric { from, to } => from <= to,
            Range::Date { from, to } => from <= to,
        }
    }

    pub fn overlap(&self, other: &Self) -> bool {
        // FIXME: need unit tests
        match (self, other) {
            (
                Range::Numeric { from, to },
                Range::Numeric {
                    from: other_from,
                    to: other_to,
                },
            ) => from <= other_to && other_from <= to,
            (
                Range::Date { from, to },
                Range::Date {
                    from: other_from,
                    to: other_to,
                },
            ) => from <= other_to && other_from <= to,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct QueryWithRange {
    pub range: Range,
    pub filters: serde_json::Value,
}

pub trait RowStream {
    fn query_range(pool: &mut PgConnection, query: &QueryWithRange) -> eyre::Result<Vec<Self>>
    where
        Self: Sized;
    fn query(pool: Arc<Mutex<PgConnection>>, queries: &[QueryWithRange]) -> impl Stream<Item = Self>
    where
        Self: Sized,
    {
        stream! {
            for query in queries {
                if !query.range.validate() {
                    log::error!("Invalid range: range={:?}", query.range);
                    continue;
                }

                let rows = Self::query_range(pool.lock().unwrap().deref_mut(), query)
                    .map_err(|e| {
                        log::error!("Error querying range: range={:?} {:?}", query, e);
                        e
                    })
                    .unwrap();
                for row in rows {
                    yield row;
                }
            }
        }
    }
}
