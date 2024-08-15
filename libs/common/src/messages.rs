use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Message {
    ProcessRange { from: Value, to: Option<Value> },
    ProcessAll,
    ProcessOne { id: Value },
    ProcessIds { ids: Vec<Value> },
}
