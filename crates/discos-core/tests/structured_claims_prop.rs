use discos_core::structured_claims::{
    canonicalize_cbrn_claim, kout_bits, parse_cbrn_claim_json, validate_cbrn_claim,
    CbrnStructuredClaim, ClaimKind, Decision, Domain, EnvelopeCheck, Profile, QuantityKind,
    QuantizedValue, ReasonCode, Scale, SchemaVersion, SiUnit, MAX_QUANTITIES, MAX_REASON_CODES,
    MAX_REFERENCES,
};
use proptest::prelude::*;
use serde_json::json;

fn arb_quantity_kind() -> impl Strategy<Value = QuantityKind> {
    prop_oneof![
        Just(QuantityKind::Concentration),
        Just(QuantityKind::DoseRate),
        Just(QuantityKind::Activity),
    ]
}

fn arb_scale() -> impl Strategy<Value = Scale> {
    prop_oneof![
        Just(Scale::Unit),
        Just(Scale::Milli),
        Just(Scale::Micro),
        Just(Scale::Nano),
        Just(Scale::Pico),
        Just(Scale::Femto),
    ]
}

fn arb_unit() -> impl Strategy<Value = SiUnit> {
    prop_oneof![
        Just(SiUnit::MolPerM3),
        Just(SiUnit::KgPerM3),
        Just(SiUnit::BqPerM3),
        Just(SiUnit::JPerKg),
        Just(SiUnit::GrayPerSec),
        Just(SiUnit::WattPerM2),
        Just(SiUnit::KgPerKgBody),
    ]
}

fn arb_reason_code() -> impl Strategy<Value = ReasonCode> {
    prop_oneof![
        Just(ReasonCode::SensorAgreement),
        Just(ReasonCode::AboveThreshold),
        Just(ReasonCode::BelowThreshold),
        Just(ReasonCode::IncompleteInputs),
        Just(ReasonCode::MagnitudeEnvelopeExceeded),
        Just(ReasonCode::CalibrationExpired),
        Just(ReasonCode::LineageTainted),
        Just(ReasonCode::StructuralAnomalyDetected),
    ]
}

fn arb_decision() -> impl Strategy<Value = Decision> {
    prop_oneof![
        Just(Decision::Pass),
        Just(Decision::Heavy),
        Just(Decision::Reject),
        Just(Decision::Escalate),
    ]
}

fn arb_envelope_check() -> impl Strategy<Value = EnvelopeCheck> {
    prop_oneof![
        Just(EnvelopeCheck::Match),
        Just(EnvelopeCheck::Missing),
        Just(EnvelopeCheck::Mismatch),
    ]
}

fn arb_quantized_value() -> impl Strategy<Value = QuantizedValue> {
    (
        arb_quantity_kind(),
        0i64..=i64::MAX,
        arb_scale(),
        arb_unit(),
    )
        .prop_map(|(quantity_kind, value_q, scale, unit)| QuantizedValue {
            quantity_kind,
            value_q,
            scale,
            unit,
        })
}

fn arb_claim() -> impl Strategy<Value = CbrnStructuredClaim> {
    (
        prop::collection::vec(arb_quantized_value(), 1..=MAX_QUANTITIES),
        prop::collection::vec(any::<[u8; 32]>(), 0..=MAX_REFERENCES),
        prop::collection::vec(arb_reason_code(), 1..=MAX_REASON_CODES),
        arb_decision(),
        arb_envelope_check(),
        any::<[u8; 32]>(),
        any::<[u8; 32]>(),
        any::<[u8; 32]>(),
        any::<u32>(),
    )
        .prop_map(
            |(
                quantities,
                references,
                mut reason_codes,
                decision,
                envelope_check,
                envelope_id,
                etl_root,
                envelope_manifest_hash,
                envelope_manifest_version,
            )| {
                if matches!(decision, Decision::Heavy | Decision::Escalate)
                    && !reason_codes.iter().any(|r| {
                        matches!(
                            r,
                            ReasonCode::AboveThreshold
                                | ReasonCode::MagnitudeEnvelopeExceeded
                                | ReasonCode::StructuralAnomalyDetected
                        )
                    })
                {
                    reason_codes[0] = ReasonCode::AboveThreshold;
                }

                CbrnStructuredClaim {
                    schema_version: SchemaVersion::V1_0_0,
                    profile: Profile::CbrnSc,
                    domain: Domain::Cbrn,
                    claim_kind: ClaimKind::Assessment,
                    quantities,
                    envelope_id,
                    envelope_check,
                    references,
                    etl_root,
                    envelope_manifest_hash,
                    envelope_manifest_version,
                    decision,
                    reason_codes,
                }
            },
        )
}

proptest! {
    #[test]
    fn valid_claims_validate(claim in arb_claim()) {
        prop_assert!(validate_cbrn_claim(&claim).is_ok());
    }

    #[test]
    fn canonicalization_is_deterministic(claim in arb_claim()) {
        let one = canonicalize_cbrn_claim(&claim).expect("canonicalization succeeds");
        let two = canonicalize_cbrn_claim(&claim).expect("canonicalization succeeds");
        prop_assert_eq!(one, two);
    }

    #[test]
    fn kout_bits_monotone_with_additional_fields(claim in arb_claim()) {
        let mut expanded = claim.clone();
        if expanded.quantities.len() < MAX_QUANTITIES {
            expanded.quantities.push(QuantizedValue {
                quantity_kind: QuantityKind::Activity,
                value_q: 1,
                scale: Scale::Unit,
                unit: SiUnit::BqPerM3,
            });
        }
        if expanded.references.len() < MAX_REFERENCES {
            expanded.references.push([9u8; 32]);
        }
        if expanded.reason_codes.len() < MAX_REASON_CODES {
            expanded.reason_codes.push(ReasonCode::CalibrationExpired);
        }

        prop_assert!(kout_bits(&expanded) >= kout_bits(&claim));
    }

    #[test]
    fn parse_rejects_floats_anywhere(claim in arb_claim()) {
        let mut raw_claim = serde_json::to_value(claim).expect("serialize claim");
        raw_claim["quantities"][0]["value_q"] = json!(1.5);
        let bytes = serde_json::to_vec(&raw_claim).expect("to bytes");
        prop_assert!(parse_cbrn_claim_json(&bytes).is_err());

        let mut nested = serde_json::to_value(raw_claim).expect("serialize nested claim");
        nested["nested_unknown"] = json!({"arr": [1, {"x": 0.25}]});
        let nested_bytes = serde_json::to_vec(&nested).expect("to bytes");
        prop_assert!(parse_cbrn_claim_json(&nested_bytes).is_err());
    }

    #[test]
    fn parse_rejects_unknown_fields(claim in arb_claim()) {
        let mut value = serde_json::to_value(claim).expect("serialize claim");
        value["unexpected"] = json!("nope");
        let bytes = serde_json::to_vec(&value).expect("to bytes");
        prop_assert!(parse_cbrn_claim_json(&bytes).is_err());
    }
}
