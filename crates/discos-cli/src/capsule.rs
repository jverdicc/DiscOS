use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PolicyOracleReceipt {
    pub oracle_id: String,
    pub decision: String,
    pub reason_code: String,
    pub wasm_hash_hex: String,
    pub manifest_hash_hex: String,
}

pub fn build_capsule_print_summary(capsule: &Value) -> Value {
    let schema = capsule
        .get("schema")
        .cloned()
        .unwrap_or(Value::String(String::new()));
    let certified = capsule.get("certified").cloned().unwrap_or(Value::Null);
    let e_value = capsule.get("e_value").cloned().unwrap_or(Value::Null);
    let decision = capsule.get("decision").cloned().unwrap_or(Value::Null);
    let reason_codes = capsule
        .get("reason_codes")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));

    let receipts = extract_policy_oracle_receipts(capsule);

    serde_json::json!({
        "capsule": {
            "schema": schema,
            "certified": certified,
            "e_value": e_value,
            "decision": decision,
            "reason_codes": reason_codes,
        },
        "policy_oracle_receipts": receipts,
    })
}

pub fn extract_policy_oracle_receipts(capsule: &Value) -> Vec<PolicyOracleReceipt> {
    let Some(receipts) = capsule
        .get("policy_oracle_receipts")
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };

    receipts
        .iter()
        .map(|receipt| PolicyOracleReceipt {
            oracle_id: string_field(receipt, "oracle_id"),
            decision: string_field(receipt, "decision"),
            reason_code: string_field(receipt, "reason_code"),
            wasm_hash_hex: string_field(receipt, "wasm_hash_hex"),
            manifest_hash_hex: string_field(receipt, "manifest_hash_hex"),
        })
        .collect()
}

fn string_field(obj: &Value, key: &str) -> String {
    obj.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned()
}
