use database::QueryWithRange;
use database::Range;
use database::Table;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
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
    pub fn push(&mut self, query: QueryWithRange) -> bool {
        if self.0.len() == 0 {
            self.0.push(query);
            return true;
        }

        let last = self.0.last().unwrap();
        match (&last.range, &query.range) {
            (Range::Numeric { .. }, Range::Numeric { .. }) => {
                for q in self.0.iter_mut() {
                    if q.range.overlap(&query.range) && q.filters == query.filters {
                        let new_range = q.range.join(&query.range);
                        q.range = new_range;
                        return true;
                    }
                }
                self.0.push(query);
                true
            }
            (Range::Date { .. }, Range::Date { .. }) => {
                for q in self.0.iter_mut() {
                    if q.range.overlap(&query.range) && q.filters == query.filters {
                        let new_range = q.range.join(&query.range);
                        q.range = new_range;
                        return true;
                    }
                }
                self.0.push(query);
                true
            }
            _ => false,
        }
    }

    pub fn lowest_ranges(&self) -> Self {
        let mut lowests: HashMap<Value, Range> = HashMap::new();
        for query in self.0.iter() {
            let key = query.filters.clone();
            if let Some(lowest) = lowests.get_mut(&key) {
                if &query.range < lowest {
                    *lowest = query.range.clone();
                }
            } else {
                lowests.insert(key, query.range.clone());
            }
        }

        let lowests = lowests
            .into_iter()
            .map(|(k, v)| QueryWithRange {
                range: v,
                filters: k,
            })
            .collect();
        ChangeSet(lowests)
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
pub enum Message {
    DataStoreUpdated {
        tier: i32,
        tables: HashMap<Table, ChangeSet>,
    },
    CancelProcessing {
        tier: i32,
        tables: Vec<Table>,
    },
}

impl Message {
    pub fn get_tier(&self) -> i32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use database::tier_1::Table as Tier1;
    use database::Table;
    use serde_json::json;

    #[test]
    fn test_view_message() {
        env_logger::try_init().ok();
        let mut tables = HashMap::new();
        let mut changes = ChangeSet::new();
        changes.push(QueryWithRange {
            range: Range::Numeric { from: 1, to: 10 },
            filters: serde_json::json!({"user": "abcde" }),
        });

        tables.insert(Table::Tier1(Tier1::Actions), changes);

        let msg = Message::DataStoreUpdated { tier: 1, tables };
        log::info!("Message: {:?}", msg);
        let actual_value = serde_json::to_value(&msg).unwrap();
        let expected_value = json!({
            "DataStoreUpdated": {
                "tier": 1,
                "tables": {
                    "transactions": [
                        {
                            "range": {
                                "numeric": {
                                    "from": 1,
                                    "to": 10
                                }
                            },
                            "filters": {
                            "user": "abcde"
                            }
                        }
                    ]
                }
            }
        });

        assert_eq!(actual_value, expected_value);

        let example_payload = r#"{
            "DataStoreUpdated": {
                "tier": 0,
                "tables": {
                    "transactions": [
                        {
                            "range": {
                                "numeric": {
                                    "from": 1,
                                    "to": 10
                                }
                            },
                            "filters": null
                        }
                    ]
                }
            }
        }"#;
        let deserialized: Message = serde_json::from_str(example_payload).unwrap();
        if let Message::DataStoreUpdated { tier, tables } = deserialized {
            assert_eq!(tier, 0);
            assert_eq!(tables.len(), 1);
            let changes = tables.get(&Table::Tier1(Tier1::Actions)).unwrap();
            assert_eq!(changes.ranges().len(), 1);
            assert_eq!(changes.ranges()[0].filters, serde_json::Value::Null);
        } else {
            panic!("Deserialization failed");
        }
    }
}
