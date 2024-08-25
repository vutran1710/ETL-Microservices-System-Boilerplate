use async_stream::stream;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
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
#[serde(rename_all = "lowercase")]
pub enum Range {
    Numeric {
        from: i64,
        to: i64,
    },
    DateTime {
        from: NaiveDateTime,
        to: NaiveDateTime,
    },
    Date {
        from: NaiveDate,
        to: NaiveDate,
    },
}

impl Range {
    fn validate(&self) -> bool {
        match self {
            Range::Numeric { from, to } => from <= to,
            Range::DateTime { from, to } => from <= to,
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
                Range::DateTime { from, to },
                Range::DateTime {
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

    pub fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (
                Range::Numeric { from, to },
                Range::Numeric {
                    from: other_from,
                    to: other_to,
                },
            ) => Range::Numeric {
                from: std::cmp::min(*from, *other_from),
                to: std::cmp::max(*to, *other_to),
            },
            (
                Range::Date { from, to },
                Range::Date {
                    from: other_from,
                    to: other_to,
                },
            ) => Range::Date {
                from: std::cmp::min(*from, *other_from),
                to: std::cmp::max(*to, *other_to),
            },
            _ => panic!("Cannot join different types of ranges"),
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
                log::info!("Get {} rows for query={:?}", rows.len(), query);
                for row in rows {
                    yield row;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_compare() {
        let d1 = chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let d2 = chrono::NaiveDate::from_ymd_opt(2021, 1, 2).unwrap();
        let d3 = chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        assert!(d1 <= d2);
        assert!(d1 <= d3);

        let r = Range::Date { from: d1, to: d2 };
        assert!(r.validate());

        let r = Range::Date { from: d1, to: d3 };
        assert!(r.validate());
    }
}
