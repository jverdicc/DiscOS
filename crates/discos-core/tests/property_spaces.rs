use discos_core::{
    boundary::{accuracy_value_det, generate_boundary},
    labels::LocalLabelsOracle,
    structured_claims::{
        validate_cbrn_claim, Analyte, CbrnStructuredClaim, Decision, QuantizedValue, ReasonCode,
        Scale, SiUnit,
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
            epoch_size: 0,
        };
        let s = TopicSignals { semantic_hash: semantic, phys_hir_signature_hash: phys, dependency_merkle_root: dep };
        let a = compute_topic_id(&m, s.clone());
        let b = compute_topic_id(&m, s);
        prop_assert_eq!(a.topic_id, b.topic_id);
    }

    #[test]
    fn structured_claim_validation_exercises_enums_and_bounds(
        analyte in prop_oneof![Just(Analyte::Nh3), Just(Analyte::Cl2), Just(Analyte::Hcn)],
        decision in prop_oneof![Just(Decision::Pass), Just(Decision::Fail), Just(Decision::Unknown)],
        scale in prop_oneof![Just(Scale::Nano), Just(Scale::Micro), Just(Scale::Milli), Just(Scale::Base)],
        unit in prop_oneof![Just(SiUnit::MolPerM3), Just(SiUnit::Ppm), Just(SiUnit::Ppb)],
        reason_count in 1usize..4,
        confidence in 0u16..=10_000,
        value_q in -10000i32..10000,
    ) {
        let reasons = vec![ReasonCode::SensorAgreement; reason_count];
        let claim = CbrnStructuredClaim {
            schema_id: "cbrn-sc.v1".into(),
            analyte,
            concentration: QuantizedValue { value_q, scale },
            unit,
            confidence_pct_x100: confidence,
            decision,
            reason_codes: reasons,
        };
        prop_assert!(validate_cbrn_claim(&claim).is_ok());
    }

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
        let obs = oracle.query_sync(&[0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1]).expect("query");
        prop_assert!(obs.bucket < num_buckets);
    }
}
