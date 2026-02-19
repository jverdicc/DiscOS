#![no_main]

use discos_core::structured_claims::{
    canonicalize_cbrn_claim, kout_bits, validate_cbrn_claim, CbrnStructuredClaim, ClaimKind,
    Decision, Domain, EnvelopeCheck, Profile, QuantityKind, QuantizedValue, ReasonCode, Scale,
    SchemaVersion, SiUnit, MAX_QUANTITIES, MAX_REASON_CODES, MAX_REFERENCES,
};
use libfuzzer_sys::{arbitrary::{Arbitrary, Unstructured}, fuzz_target};

#[derive(Arbitrary, Debug)]
struct FuzzClaim {
    quantities: Vec<FuzzQuantity>,
    references: Vec<[u8; 32]>,
    reason_codes: Vec<FuzzReasonCode>,
    envelope_id: [u8; 32],
    envelope_check: FuzzEnvelopeCheck,
    etl_root: [u8; 32],
    envelope_manifest_hash: [u8; 32],
    envelope_manifest_version: u32,
    decision: FuzzDecision,
}

#[derive(Arbitrary, Debug)]
struct FuzzQuantity {
    quantity_kind: FuzzQuantityKind,
    value_q: i64,
    scale: FuzzScale,
    unit: FuzzSiUnit,
}

#[derive(Arbitrary, Debug)]
enum FuzzQuantityKind { Concentration, DoseRate, Activity }
#[derive(Arbitrary, Debug)]
enum FuzzScale { Unit, Milli, Micro, Nano, Pico, Femto }
#[derive(Arbitrary, Debug)]
enum FuzzSiUnit { MolPerM3, KgPerM3, BqPerM3, JPerKg, GrayPerSec, WattPerM2, KgPerKgBody }
#[derive(Arbitrary, Debug)]
enum FuzzDecision { Pass, Heavy, Reject, Escalate }
#[derive(Arbitrary, Debug)]
enum FuzzEnvelopeCheck { Match, Missing, Mismatch }
#[derive(Arbitrary, Debug)]
enum FuzzReasonCode {
    SensorAgreement,
    AboveThreshold,
    BelowThreshold,
    IncompleteInputs,
    MagnitudeEnvelopeExceeded,
    CalibrationExpired,
    LineageTainted,
    StructuralAnomalyDetected,
}

impl From<FuzzQuantityKind> for QuantityKind { fn from(v: FuzzQuantityKind) -> Self { match v { FuzzQuantityKind::Concentration => Self::Concentration, FuzzQuantityKind::DoseRate => Self::DoseRate, FuzzQuantityKind::Activity => Self::Activity } } }
impl From<FuzzScale> for Scale { fn from(v: FuzzScale) -> Self { match v { FuzzScale::Unit => Self::Unit, FuzzScale::Milli => Self::Milli, FuzzScale::Micro => Self::Micro, FuzzScale::Nano => Self::Nano, FuzzScale::Pico => Self::Pico, FuzzScale::Femto => Self::Femto } } }
impl From<FuzzSiUnit> for SiUnit { fn from(v: FuzzSiUnit) -> Self { match v { FuzzSiUnit::MolPerM3 => Self::MolPerM3, FuzzSiUnit::KgPerM3 => Self::KgPerM3, FuzzSiUnit::BqPerM3 => Self::BqPerM3, FuzzSiUnit::JPerKg => Self::JPerKg, FuzzSiUnit::GrayPerSec => Self::GrayPerSec, FuzzSiUnit::WattPerM2 => Self::WattPerM2, FuzzSiUnit::KgPerKgBody => Self::KgPerKgBody } } }
impl From<FuzzDecision> for Decision { fn from(v: FuzzDecision) -> Self { match v { FuzzDecision::Pass => Self::Pass, FuzzDecision::Heavy => Self::Heavy, FuzzDecision::Reject => Self::Reject, FuzzDecision::Escalate => Self::Escalate } } }
impl From<FuzzEnvelopeCheck> for EnvelopeCheck { fn from(v: FuzzEnvelopeCheck) -> Self { match v { FuzzEnvelopeCheck::Match => Self::Match, FuzzEnvelopeCheck::Missing => Self::Missing, FuzzEnvelopeCheck::Mismatch => Self::Mismatch } } }
impl From<FuzzReasonCode> for ReasonCode { fn from(v: FuzzReasonCode) -> Self { match v { FuzzReasonCode::SensorAgreement => Self::SensorAgreement, FuzzReasonCode::AboveThreshold => Self::AboveThreshold, FuzzReasonCode::BelowThreshold => Self::BelowThreshold, FuzzReasonCode::IncompleteInputs => Self::IncompleteInputs, FuzzReasonCode::MagnitudeEnvelopeExceeded => Self::MagnitudeEnvelopeExceeded, FuzzReasonCode::CalibrationExpired => Self::CalibrationExpired, FuzzReasonCode::LineageTainted => Self::LineageTainted, FuzzReasonCode::StructuralAnomalyDetected => Self::StructuralAnomalyDetected } } }

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(mut f) = FuzzClaim::arbitrary(&mut u) {
        f.quantities.truncate(MAX_QUANTITIES);
        f.references.truncate(MAX_REFERENCES);
        f.reason_codes.truncate(MAX_REASON_CODES);

        if f.quantities.is_empty() {
            f.quantities.push(FuzzQuantity {
                quantity_kind: FuzzQuantityKind::Concentration,
                value_q: 0,
                scale: FuzzScale::Unit,
                unit: FuzzSiUnit::MolPerM3,
            });
        }
        if f.reason_codes.is_empty() {
            f.reason_codes.push(FuzzReasonCode::SensorAgreement);
        }

        let claim = CbrnStructuredClaim {
            schema_version: SchemaVersion::V1_0_0,
            profile: Profile::CbrnSc,
            domain: Domain::Cbrn,
            claim_kind: ClaimKind::Assessment,
            quantities: f.quantities.into_iter().map(|q| QuantizedValue {
                quantity_kind: q.quantity_kind.into(),
                value_q: q.value_q.max(0),
                scale: q.scale.into(),
                unit: q.unit.into(),
            }).collect(),
            envelope_id: f.envelope_id,
            envelope_check: f.envelope_check.into(),
            references: f.references,
            etl_root: f.etl_root,
            envelope_manifest_hash: f.envelope_manifest_hash,
            envelope_manifest_version: f.envelope_manifest_version,
            decision: f.decision.into(),
            reason_codes: f.reason_codes.into_iter().map(Into::into).collect(),
        };

        let _ = validate_cbrn_claim(&claim);
        let _ = canonicalize_cbrn_claim(&claim);
        let _ = kout_bits(&claim);
    }
});
