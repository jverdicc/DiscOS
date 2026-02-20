use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn demo_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/exfiltration_demo/attack_bitflip.py")
}

fn run_demo(mode: &str) -> Value {
    let output = Command::new("python3")
        .arg(demo_script())
        .arg("--mode")
        .arg(mode)
        .arg("--n")
        .arg("64")
        .arg("--seed")
        .arg("7")
        .arg("--format")
        .arg("json")
        .output()
        .expect("python3 should execute exfiltration demo script");

    assert!(
        output.status.success(),
        "demo should succeed for mode {mode}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("demo should emit valid JSON")
}

#[test]
fn baseline_oracle_leaks_with_high_recovery_accuracy() {
    let result = run_demo("baseline");
    let recovered = result["recovered_accuracy"]
        .as_f64()
        .expect("recovered_accuracy should be f64");

    assert!(
        recovered >= 0.95,
        "expected baseline leakage >= 0.95, got {recovered}"
    );
}

#[test]
fn evidenceos_mock_mitigation_keeps_recovery_low() {
    let result = run_demo("evidenceos-mock");
    let recovered = result["recovered_accuracy"]
        .as_f64()
        .expect("recovered_accuracy should be f64");

    assert!(
        recovered <= 0.60,
        "expected EvidenceOS-style mitigation <= 0.60, got {recovered}"
    );
}
