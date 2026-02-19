#[cfg(feature = "sim")]
use discos_core::{
    boundary::{accuracy_value_det, generate_boundary},
    labels::{AccuracyOracle, LocalLabelsOracle},
};
use discos_core::{
    structured_claims::{
        validate_cbrn_claim, CbrnStructuredClaim, ClaimKind, Decision, Domain, EnvelopeCheck,
        Profile, QuantityKind, QuantizedValue, ReasonCode, Scale, SchemaVersion, SiUnit,
    },
    topicid::{compute_topic_id, ClaimMetadata, TopicSignals},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn topicid_deterministic_over_parameter_space(
        lane in "[a-z0-9_/-]{1,32}",
        alpha in 0u32..1_000_000,
        epoch in "[a-zA-Z0-9_.:-]{1,32}",
        schema in "[a-zA-Z0-9_.:-]{1,32}",
        semantic in any::<Option<[u8;32]>>(),
        phys in any::<[u8;32]>(),
        dep in any::<Option<[u8;32]>>()
    ) {
        let m = ClaimMetadata {
            lane,
            alpha_micros: alpha,
            epoch_config_ref: epoch,
            output_schema_id: schema,
        };
        let s = TopicSignals { semantic_hash: semantic, phys_hir_signature_hash: phys, dependency_merkle_root: dep };
        let a = compute_topic_id(&m, s.clone());
        let b = compute_topic_id(&m, s);
        prop_assert_eq!(a.topic_id, b.topic_id);
    }

    #[test]
    fn structured_claim_validation_exercises_enums_and_bounds(
        decision in prop_oneof![Just(Decision::Pass), Just(Decision::Heavy), Just(Decision::Reject), Just(Decision::Escalate)],
        scale in prop_oneof![Just(Scale::Nano), Just(Scale::Micro), Just(Scale::Milli), Just(Scale::Unit)],
        unit in prop_oneof![Just(SiUnit::MolPerM3), Just(SiUnit::BqPerM3), Just(SiUnit::KgPerM3)],
        reason_count in 1usize..4,
        value_q in 0i64..10000,
    ) {
        let reasons = if matches!(decision, Decision::Heavy | Decision::Escalate) {
            vec![ReasonCode::AboveThreshold; reason_count]
        } else {
            vec![ReasonCode::SensorAgreement; reason_count]
        };
        let claim = CbrnStructuredClaim {
            schema_version: SchemaVersion::V1_0_0,
            profile: Profile::CbrnSc,
            domain: Domain::Cbrn,
            claim_kind: ClaimKind::Assessment,
            quantities: vec![QuantizedValue { quantity_kind: QuantityKind::Concentration, value_q, scale, unit }],
            envelope_id: [1u8; 32],
            envelope_check: EnvelopeCheck::Match,
            references: vec![[2u8; 32]],
            etl_root: [3u8; 32],
            envelope_manifest_hash: [4u8; 32],
            envelope_manifest_version: 1,
            decision,
            reason_codes: reasons,
        };
        prop_assert!(validate_cbrn_claim(&claim).is_ok());
    }

    #[cfg(feature = "sim")]
    #[test]
    fn boundary_and_labels_oracles_cover_ranges(
        seed in any::<u64>(),
        delta_sigma in 0.0f64..0.5,
        num_buckets in 2u32..1024,
    ) {
        let b = generate_boundary(seed);
        let a = accuracy_value_det(0.5, b);
        prop_assert!((0.0..=1.0).contains(&a));

        let labels = vec![0u8,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1];
        let mut oracle = LocalLabelsOracle::new(labels, num_buckets, delta_sigma).expect("labels oracle");
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("rt");
        let obs = runtime.block_on(oracle.query_accuracy(&[0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1])).expect("query");
        prop_assert!(obs.bucket < num_buckets);
    }
}
