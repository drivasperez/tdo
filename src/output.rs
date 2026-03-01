// r[output.tsv]
// r[output.json]
// r[output.tsv.fields]
// r[output.no-header]
// r[test.output-formats]

use crate::error::Error;
use crate::model::{KeyValue, Row};

/// Whether to print the TSV header row.
pub enum Header {
    Show,
    Hide,
}

/// Bundled parameters for `print_tsv`.
pub struct TsvConfig<'a> {
    pub default_fields: &'a [&'a str],
    pub fields: &'a Option<String>,
    pub header: Header,
}

/// Print rows as TSV with optional field filtering and header suppression.
pub fn print_tsv(rows: &[Row], config: &TsvConfig) -> Result<(), Error> {
    let columns: Vec<&str> = match config.fields {
        Some(f) => f.split(',').map(|s| s.trim()).collect(),
        None => config.default_fields.to_vec(),
    };

    if matches!(config.header, Header::Show) {
        println!("{}", columns.join("\t"));
    }

    for row in rows {
        let values: Vec<String> = columns.iter().map(|col| row.get_str(col)).collect();
        println!("{}", values.join("\t"));
    }
    Ok(())
}

/// Print rows as JSON array (all fields included).
pub fn print_json(rows: &[Row]) -> Result<(), Error> {
    let output: Vec<&std::collections::BTreeMap<String, serde_json::Value>> =
        rows.iter().map(|r| &r.0).collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print key-value pairs as TSV.
pub fn print_kv_tsv(kvs: &[KeyValue], header: Header) -> Result<(), Error> {
    if matches!(header, Header::Show) {
        println!("key\tvalue");
    }
    for kv in kvs {
        let val = match &kv.value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        println!("{}\t{val}", kv.key);
    }
    Ok(())
}

/// Print key-value pairs as JSON object.
pub fn print_kv_json(kvs: &[KeyValue]) -> Result<(), Error> {
    let map: serde_json::Map<String, serde_json::Value> = kvs
        .iter()
        .map(|kv| (kv.key.clone(), kv.value.clone()))
        .collect();
    println!("{}", serde_json::to_string_pretty(&map)?);
    Ok(())
}

/// Print a single item as key-value TSV pairs.
pub fn print_show_tsv(row: &Row, header: Header) -> Result<(), Error> {
    let kvs: Vec<KeyValue> = row
        .0
        .iter()
        .map(|(k, v)| KeyValue {
            key: k.clone(),
            value: v.clone(),
        })
        .collect();
    // For show, we skip the "key\tvalue" header since it's implicit
    if matches!(header, Header::Show) {
        println!("key\tvalue");
    }
    for kv in &kvs {
        let val = match &kv.value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            serde_json::Value::Array(arr) => {
                // Format arrays as comma-separated
                arr.iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            }
            other => other.to_string(),
        };
        println!("{}\t{val}", kv.key);
    }
    Ok(())
}

/// Print a single item as JSON object.
pub fn print_show_json(row: &Row) -> Result<(), Error> {
    println!("{}", serde_json::to_string_pretty(&row.0)?);
    Ok(())
}
