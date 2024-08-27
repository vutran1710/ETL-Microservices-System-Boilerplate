use database::RangeQuery;
use database::Table;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    DataStoreUpdated { table: Table, range: RangeQuery },
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::DataStoreUpdated { table, range } => {
                write!(
                    f,
                    "** DataStoreUpdated: table: {:?}, range: {:?}",
                    table, range
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::tier_1::Table as Tier1;
    use database::{Range, Table};
    use serde_json::json;

    #[test]
    fn test_view_message() {
        env_logger::try_init().ok();
        let range = RangeQuery {
            range: Range::Numeric { from: 1, to: 10 },
            filters: serde_json::json!({"user": "abcde" }),
        };

        let msg = Message::DataStoreUpdated {
            table: Table::Tier1(Tier1::Actions),
            range,
        };
        log::info!("Message: {:?}", msg);
        let actual_value = serde_json::to_value(&msg).unwrap();
        let expected_value = json!({
            "DataStoreUpdated": {
                "table": "actions",
                "range": {
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
            }
        });

        assert_eq!(actual_value, expected_value);

        let example_payload = r#"{
            "DataStoreUpdated": {
                "table": "actions",
                "range": {
                    "range": {
                        "numeric": {
                            "from": 1,
                            "to": 10
                        }
                    },
                    "filters": null
                }
            }
        }"#;
        let deserialized: Message = serde_json::from_str(example_payload).unwrap();
        #[allow(irrefutable_let_patterns)]
        if let Message::DataStoreUpdated { table, range } = deserialized {
            assert_eq!(table, Table::Tier1(Tier1::Actions));
            assert_eq!(range.filters, serde_json::Value::Null);
        } else {
            panic!("Deserialization failed");
        }
    }
}
