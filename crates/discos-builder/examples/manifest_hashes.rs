use discos_builder::{manifest_hash, AlphaHIRManifest};

fn main() {
    let manifest = AlphaHIRManifest {
        plan_id: "example-plan".into(),
        code_hash_hex: "00".repeat(32),
        oracle_kinds: vec!["oracle_query".into()],
        output_schema_id: "cbrn-sc.v1".into(),
        nullspec_id: "nullspec.v1".into(),
    };
    let hash = manifest_hash(&manifest).expect("manifest hash");
    println!("manifest_hash={:02x?}", hash);
}
