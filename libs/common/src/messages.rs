use database::interfaces::OrderingID;
use database::interfaces::RangeID;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
pub struct Change<T: OrderingID> {
    pub version: i64,
    pub range: RangeID<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ChangeSet<T: OrderingID>(Vec<Change<T>>);

impl<T: OrderingID> ChangeSet<T> {
    pub fn new(changes: Vec<Change<T>>) -> Self {
        Self(changes)
    }

    /// ChangeSet must not have overlapping ranges
    pub fn validate(&self) -> bool {
        let mut sorted = self.0.clone();
        sorted.sort_by(|a, b| a.range.from.cmp(&b.range.from));
        for i in 0..sorted.len() - 1 {
            if sorted[i].range.overlap(&sorted[i + 1].range) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Message<T: OrderingID> {
    DataStoreUpdated {
        tier: i64,
        tables: HashMap<String, ChangeSet<T>>,
    },
}
