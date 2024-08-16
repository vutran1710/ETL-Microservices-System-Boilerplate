use diesel::PgConnection;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use async_stream::stream;

use futures_core::stream::Stream;

/// Index that is used to order the changes
pub trait OrderingID:
    Debug + Serialize + Ord + PartialOrd + Eq + PartialEq + Sized + Clone + Send + Sync + 'static
{
}

/// Range is [from, to]: both are inclusive
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Copy, PartialOrd, Ord)]
pub struct RangeID<T: OrderingID> {
    pub from: T,
    pub to: T,
}

impl<T: OrderingID> RangeID<T> {
    pub fn new(from: T, to: T) -> eyre::Result<Self> {
        let this = Self { from, to };
        if !this.validate() {
            eyre::bail!("Invalid range: {:?}", this);
        }
        Ok(this)
    }

    fn validate(&self) -> bool {
        self.from < self.to
    }

    pub fn overlap(&self, other: &Self) -> bool {
        if self.to < other.from {
            // The other range is after this range
            return false;
        }
        if self.from > other.to {
            // The other range is before this range
            return false;
        }
        true
    }
}

pub trait RowStream<T: OrderingID> {
    fn query_range(pool: &mut PgConnection, range: &RangeID<T>) -> eyre::Result<Vec<Self>>
    where
        Self: Sized;
    fn query(pool: Arc<Mutex<PgConnection>>, ranges: &[RangeID<T>]) -> impl Stream<Item = Self>
    where
        Self: Sized,
    {
        stream! {
            for range in ranges {
                let rows = Self::query_range(pool.lock().unwrap().deref_mut(), range)
                    .map_err(|e| {
                        log::error!("Error querying range: range={:?} {:?}", range, e);
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
