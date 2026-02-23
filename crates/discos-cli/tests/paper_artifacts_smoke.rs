use std::process::Command;

#[test]
fn paper_artifacts_wrapper_requires_evidenceos_authoritative_runner() {
    let output = Command::new("python3")
        .arg("paper_artifacts/reproduce_paper.py")
        .arg("--evidenceos-repo")
        .arg("/definitely/missing/evidenceos")
        .output()
        .expect("run wrapper");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("authoritative paper reproduction runner not found"));
}
