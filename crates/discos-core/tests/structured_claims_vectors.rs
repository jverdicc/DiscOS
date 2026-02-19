use std::{fs, path::Path};

use discos_core::structured_claims::{
    canonicalize_cbrn_claim, parse_cbrn_claim_json, validate_cbrn_claim,
};

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn valid_vectors_parse_validate_and_match_golden() {
    let valid_dir = Path::new("test_vectors/structured_claims/valid");
    for entry in fs::read_dir(valid_dir).expect("read valid dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let bytes = read(&path);
        let claim = parse_cbrn_claim_json(&bytes)
            .unwrap_or_else(|e| panic!("{} should parse: {e}", path.display()));
        validate_cbrn_claim(&claim)
            .unwrap_or_else(|e| panic!("{} should validate: {e}", path.display()));
        let canonical = canonicalize_cbrn_claim(&claim)
            .unwrap_or_else(|e| panic!("{} should canonicalize: {e}", path.display()));

        let golden_path = path.with_extension("hex");
        let expected_hex = fs::read_to_string(&golden_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", golden_path.display()));
        assert_eq!(
            hex::encode(canonical),
            expected_hex.trim(),
            "{}",
            path.display()
        );
    }
}

#[test]
fn invalid_vectors_fail_parse_or_validation() {
    let invalid_dir = Path::new("test_vectors/structured_claims/invalid");
    for entry in fs::read_dir(invalid_dir).expect("read invalid dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let bytes = read(&path);
        if let Ok(claim) = parse_cbrn_claim_json(&bytes) {
            assert!(
                validate_cbrn_claim(&claim).is_err(),
                "{} unexpectedly validated",
                path.display()
            );
        }
    }
}
