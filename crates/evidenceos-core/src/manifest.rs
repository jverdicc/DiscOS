use serde::Serialize;
use serde_json::{Map, Value};
use std::io;

fn write_canonical(value: &Value, out: &mut String) -> Result<(), serde_json::Error> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
        Value::Number(v) => out.push_str(&v.to_string()),
        Value::String(v) => out.push_str(&serde_json::to_string(v)?),
        Value::Array(values) => {
            out.push('[');
            for (idx, entry) in values.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_canonical(entry, out)?;
            }
            out.push(']');
        }
        Value::Object(entries) => write_canonical_object(entries, out)?,
    }
    Ok(())
}

fn write_canonical_object(
    entries: &Map<String, Value>,
    out: &mut String,
) -> Result<(), serde_json::Error> {
    let mut keys: Vec<&str> = entries.keys().map(String::as_str).collect();
    keys.sort_unstable();

    out.push('{');
    for (idx, key) in keys.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&serde_json::to_string(key)?);
        out.push(':');
        let entry = entries.get(*key).ok_or_else(|| {
            serde_json::Error::io(io::Error::new(
                io::ErrorKind::InvalidData,
                "canonical JSON key lookup failed",
            ))
        })?;
        write_canonical(entry, out)?;
    }
    out.push('}');
    Ok(())
}

pub fn canonical_json_string<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    let mut out = String::new();
    write_canonical(&value, &mut out)?;
    Ok(out)
}

pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    Ok(canonical_json_string(value)?.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_orders_object_keys_recursively() {
        let value =
            serde_json::json!({"b": 2, "a": {"z": 1, "c": true}, "arr": [{"k2": 2, "k1":1}]});
        let canonical = canonical_json_string(&value).expect("canonical json");
        assert_eq!(
            canonical,
            r#"{"a":{"c":true,"z":1},"arr":[{"k1":1,"k2":2}],"b":2}"#
        );
    }

    #[test]
    fn canonical_json_bytes_matches_canonical_string_encoding() {
        let value = serde_json::json!({"b": 2, "a": 1});
        let canonical_string = canonical_json_string(&value).expect("canonical string");
        let canonical_bytes = canonical_json_bytes(&value).expect("canonical bytes");
        assert_eq!(canonical_bytes, canonical_string.into_bytes());
    }
}
