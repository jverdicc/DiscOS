use std::fs;

#[test]
fn readme_uses_expected_evidenceos_repository_urls() {
    let readme_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    let readme = fs::read_to_string(&readme_path).expect("read README.md");

    assert!(
        readme.contains("https://github.com/jverdicc/EvidenceOS"),
        "README should reference the canonical EvidenceOS repository URL"
    );

    for legacy in [
        "https://github.com/EvidenceOS/evidenceos",
        "https://github.com/evidenceos/evidenceos",
    ] {
        assert!(
            !readme.contains(legacy),
            "README still contains legacy EvidenceOS URL: {legacy}"
        );
    }
}
