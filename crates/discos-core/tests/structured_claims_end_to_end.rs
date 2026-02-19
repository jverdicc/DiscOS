use discos_core::structured_claims::{
    canonicalize_cbrn_claim, kout_accounting, kout_budget_charge, parse_cbrn_claim_json,
    validate_cbrn_claim, CbrnStructuredClaim, ClaimKind, Decision, Domain, EnvelopeCheck, Profile,
    QuantityKind, QuantizedValue, ReasonCode, Scale, SchemaVersion, SiUnit,
};
use evidenceos_core::{
    etl::{verify_inclusion_proof_ct, Etl},
    ledger::ConservationLedger,
};

#[test]
fn end_to_end_claim_roundtrip_and_kout_and_etl() {
    let claim = CbrnStructuredClaim {
        schema_version: SchemaVersion::V1_0_0,
        profile: Profile::CbrnSc,
        domain: Domain::Cbrn,
        claim_kind: ClaimKind::Assessment,
        quantities: vec![
            QuantizedValue {
                quantity_kind: QuantityKind::Concentration,
                value_q: 12_345,
                scale: Scale::Micro,
                unit: SiUnit::MolPerM3,
            },
            QuantizedValue {
                quantity_kind: QuantityKind::DoseRate,
                value_q: 88,
                scale: Scale::Milli,
                unit: SiUnit::GrayPerSec,
            },
        ],
        envelope_id: [7u8; 32],
        envelope_check: EnvelopeCheck::Match,
        references: vec![[9u8; 32], [10u8; 32]],
        etl_root: [11u8; 32],
        envelope_manifest_hash: [12u8; 32],
        envelope_manifest_version: 42,
        decision: Decision::Heavy,
        reason_codes: vec![ReasonCode::AboveThreshold, ReasonCode::SensorAgreement],
    };

    let json = serde_json::to_vec(&claim).expect("serialize claim to json");
    let parsed = parse_cbrn_claim_json(&json).expect("parse json claim");

    validate_cbrn_claim(&parsed).expect("claim validates");

    let canonical_a = canonicalize_cbrn_claim(&parsed).expect("canonicalization succeeds");
    let canonical_b = canonicalize_cbrn_claim(&parsed).expect("canonicalization deterministic");
    assert_eq!(canonical_a, canonical_b);

    let accounting = kout_accounting(&parsed);
    assert!(accounting.kout_bits > 0);
    let charge_bits = kout_budget_charge(&parsed);
    assert_eq!(charge_bits, accounting.kout_bits as f64);

    let mut ledger = ConservationLedger::new(charge_bits + 10.0);
    let remaining = ledger
        .charge(charge_bits)
        .expect("ledger charge should succeed");
    assert_eq!(ledger.charged_bits(), charge_bits);
    assert_eq!(remaining, ledger.budget_bits() - charge_bits);

    let temp = tempfile::tempdir().expect("tempdir");
    let mut etl = Etl::new(temp.path()).expect("create etl");
    let (_idx, proof) = etl.append(&canonical_a).expect("append to etl");
    let root = etl.root().expect("etl root exists");

    assert!(verify_inclusion_proof_ct(root, &proof));

    let mut tampered = proof.clone();
    if tampered.audit_path.is_empty() {
        tampered.audit_path.push([0u8; 32]);
    }
    tampered.audit_path[0][0] ^= 0x01;
    assert!(!verify_inclusion_proof_ct(root, &tampered));
}
