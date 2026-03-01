use serde::Serialize;
use std::collections::BTreeMap;

// r[fields.id]
// r[test.fixtures]

/// A generic row of key-value data from a query.
/// Uses BTreeMap for stable ordering.
#[derive(Debug, Clone, Serialize)]
pub struct Row(pub BTreeMap<String, serde_json::Value>);

impl Row {
    pub fn new() -> Self {
        Row(BTreeMap::new())
    }

    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.0.insert(key.to_string(), value);
    }

    pub fn get_str(&self, key: &str) -> String {
        match self.0.get(key) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::Bool(b)) => b.to_string(),
            Some(serde_json::Value::Null) | None => String::new(),
            Some(v) => v.to_string(),
        }
    }
}

/// Key-value pair for show/stats output.
#[derive(Debug, Clone, Serialize)]
pub struct KeyValue {
    pub key: String,
    pub value: serde_json::Value,
}

/// Checklist item for show output.
#[derive(Debug, Clone, Serialize)]
pub struct ChecklistItem {
    pub title: String,
    pub status: i64,
}
