use std::fs;

use discos_cli::artifacts::run_paper_suite;

#[tokio::test]
async fn minimal_paper_suite_subset_writes_reproducible_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("paper-suite");

    let index = run_paper_suite(&out, "http://127.0.0.1:50051")
        .await
        .expect("paper suite run");

    assert_eq!(index.schema_version, "discos.paper-suite.index.v1");
    assert!(out.join("index.json").exists());
    assert!(out.join("exp11.json").exists());
    assert!(out.join("exp12.json").exists());
    assert!(out.join("canary_drift.json").exists());
    assert!(out.join("multisignal_topicid.json").exists());

    let index_json = fs::read_to_string(out.join("index.json")).expect("read index");
    assert!(index_json.contains("discos.paper-suite.index.v1"));
}
