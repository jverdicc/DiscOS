use std::process::Command;

#[test]
fn paper_artifacts_smoke_subset_generates_expected_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("paper-artifacts-smoke");

    let status = Command::new("python3")
        .arg("paper_artifacts/reproduce_paper.py")
        .arg("--smoke")
        .arg("--out")
        .arg(&out)
        .status()
        .expect("run smoke artifact generator");

    assert!(status.success());
    assert!(out.join("index.json").exists());
    assert!(out.join("exp01.json").exists());
    assert!(out.join("exp11.json").exists());
    assert!(out.join("exp12.json").exists());
}
