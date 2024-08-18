use database::QueryWithRange;
use database::Range;
use database::Table;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ChangeSet(Vec<QueryWithRange>);

impl ChangeSet {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn ranges(&self) -> &Vec<QueryWithRange> {
        &self.0
    }

    // NOTE: add a range to the ChangeSet
    // But only accept range of same type
    // FIXME: join the ranges that are adjacent
    pub fn push(&mut self, query: QueryWithRange) -> bool {
        if self.0.len() == 0 {
            self.0.push(query);
            return true;
        }

        let last = self.0.last().unwrap();
        match (&last.range, &query.range) {
            (Range::Numeric { .. }, Range::Numeric { .. }) => {
                self.0.push(query);
                true
            }
            (Range::Date { .. }, Range::Date { .. }) => {
                self.0.push(query);
                true
            }
            _ => false,
        }
        // FIXME: check for overlapping ranges
    }

    /// ChangeSet must not have overlapping ranges
    pub fn validate(&self) -> bool {
        let mut sorted = self.0.clone();
        sorted.sort_by(|a, b| a.range.cmp(&b.range));
        for i in 0..sorted.len() - 1 {
            if sorted[i].range.overlap(&sorted[i + 1].range) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all_fields = "SCREAMING_SNAKE_CASE")]
pub enum Message {
    DataStoreUpdated {
        tier: i64,
        tables: HashMap<Table, ChangeSet>,
    },
    CancelProcessing {
        tier: i64,
        tables: Vec<Table>,
    },
}

impl Message {
    pub fn get_tier(&self) -> i64 {
        match self {
            Message::DataStoreUpdated { tier, .. } => *tier,
            Message::CancelProcessing { tier, .. } => *tier,
        }
    }
}

impl std::fmt::Display for Message {
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
