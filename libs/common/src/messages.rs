use database::OrderingID;
use database::RangeID;
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
    pub fn get_change_ranges(&self) -> Vec<RangeID<T>> {
        self.0.iter().map(|c| c.range.clone()).collect()
    }
}

impl<T: OrderingID> ChangeSet<T> {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_change(&mut self, change: Change<T>) {
        self.0.push(change);
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
    CancelProcessing {
        tier: i64,
        tables: Vec<String>,
    },
}

impl<T: OrderingID> Message<T> {
    pub fn get_tier(&self) -> i64 {
        match self {
            Message::DataStoreUpdated { tier, .. } => *tier,
            Message::CancelProcessing { tier, .. } => *tier,
        }
    }
}

impl<T: OrderingID> std::fmt::Display for Message<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::DataStoreUpdated { tier, tables } => {
                write!(
                    f,
                    "** DataStoreUpdated: tier: {}, tables: {:?}",
                    tier, tables
                )
            }
            Message::CancelProcessing { tier, tables } => {
                write!(
                    f,
                    "** CancelProcessing: tier: {}, tables: {:?}",
                    tier, tables
                )
            }
        }
    }
}
