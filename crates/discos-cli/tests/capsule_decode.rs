use discos_cli::capsule::{
    build_capsule_print_summary, extract_oracle_metadata, extract_policy_oracle_receipts,
};

#[test]
fn extracts_and_formats_policy_oracle_receipts() {
    let capsule = serde_json::json!({
        "schema": "evidenceos.claim-capsule.v1",
        "certified": true,
        "e_value": 0.2,
        "decision": "defer",
        "reason_codes": ["SJ_DEFER"],
        "oracle_id": "acme.safety.v1",
        "oracle_resolution_hash": "cc33",
        "oracle_manifest_hash": "dd44",
        "policy_oracle_receipts": [
            {
                "oracle_id": "super-judge-1",
                "decision": "veto",
                "reason_code": "SJ_VETO",
                "wasm_hash_hex": "aa11",
                "manifest_hash_hex": "bb22"
            }
        ]
    });

    let receipts = extract_policy_oracle_receipts(&capsule);
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].oracle_id, "super-judge-1");

    let oracle = extract_oracle_metadata(&capsule);
    assert_eq!(oracle.oracle_id, "acme.safety.v1");
    assert_eq!(oracle.oracle_resolution_hash, "cc33");
    assert_eq!(oracle.oracle_manifest_hash, "dd44");

    let summary = build_capsule_print_summary(&capsule);
    assert_eq!(summary["capsule"]["schema"], "evidenceos.claim-capsule.v1");
    assert_eq!(summary["capsule"]["decision"], "defer");
    assert_eq!(summary["oracle"]["oracle_id"], "acme.safety.v1");
    assert_eq!(
        summary["policy_oracle_receipts"][0]["reason_code"],
        "SJ_VETO"
    );
}

#[test]
fn missing_policy_oracle_receipts_is_backward_compatible() {
    let capsule = serde_json::json!({
        "schema": "evidenceos.claim-capsule.v1",
        "certified": true,
        "decision": "allow"
    });

    let receipts = extract_policy_oracle_receipts(&capsule);
    assert!(receipts.is_empty());

    let summary = build_capsule_print_summary(&capsule);
    assert_eq!(summary["policy_oracle_receipts"], serde_json::json!([]));
    assert_eq!(summary["oracle"]["oracle_id"], "");
    assert_eq!(summary["oracle"]["oracle_resolution_hash"], "");
    assert_eq!(summary["oracle"]["oracle_manifest_hash"], "");
}
