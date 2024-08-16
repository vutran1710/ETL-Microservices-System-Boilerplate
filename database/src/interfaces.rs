use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

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
