// Copyright 2026 Joseph Verdicchio
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};

const MAX_REASON_CODES: usize = 8;
const MAX_REFERENCES: usize = 16;
const MAX_QUANTITIES: usize = 8;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaVersion {
    V1_0_0,
}

impl SchemaVersion {
    pub const fn variant_count() -> usize {
        1
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::V1_0_0 => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    CbrnSc,
}

impl Profile {
    pub const fn variant_count() -> usize {
        1
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::CbrnSc => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Cbrn,
}

impl Domain {
    pub const fn variant_count() -> usize {
        1
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Cbrn => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Assessment,
}

impl ClaimKind {
    pub const fn variant_count() -> usize {
        1
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Assessment => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuantityKind {
    Concentration,
    DoseRate,
    Activity,
}

impl QuantityKind {
    pub const fn variant_count() -> usize {
        3
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Concentration => 0,
            Self::DoseRate => 1,
            Self::Activity => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scale {
    Unit,
    Milli,
    Micro,
    Nano,
    Pico,
    Femto,
}

impl Scale {
    pub const fn variant_count() -> usize {
        6
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Unit => 0,
            Self::Milli => 1,
            Self::Micro => 2,
            Self::Nano => 3,
            Self::Pico => 4,
            Self::Femto => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SiUnit {
    MolPerM3,
    KgPerM3,
    BqPerM3,
    JPerKg,
    GrayPerSec,
    WattPerM2,
    KgPerKgBody,
}

impl SiUnit {
    pub const fn variant_count() -> usize {
        7
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::MolPerM3 => 0,
            Self::KgPerM3 => 1,
            Self::BqPerM3 => 2,
            Self::JPerKg => 3,
            Self::GrayPerSec => 4,
            Self::WattPerM2 => 5,
            Self::KgPerKgBody => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Pass,
    Heavy,
    Reject,
    Escalate,
}

impl Decision {
    pub const fn variant_count() -> usize {
        4
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Heavy => 1,
            Self::Reject => 2,
            Self::Escalate => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    SensorAgreement,
    AboveThreshold,
    BelowThreshold,
    IncompleteInputs,
    MagnitudeEnvelopeExceeded,
    CalibrationExpired,
    LineageTainted,
    StructuralAnomalyDetected,
}

impl ReasonCode {
    pub const fn variant_count() -> usize {
        8
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::SensorAgreement => 0,
            Self::AboveThreshold => 1,
            Self::BelowThreshold => 2,
            Self::IncompleteInputs => 3,
            Self::MagnitudeEnvelopeExceeded => 4,
            Self::CalibrationExpired => 5,
            Self::LineageTainted => 6,
            Self::StructuralAnomalyDetected => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeCheck {
    Match,
    Missing,
    Mismatch,
}

impl EnvelopeCheck {
    pub const fn variant_count() -> usize {
        3
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Match => 0,
            Self::Missing => 1,
            Self::Mismatch => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuantizedValue {
    pub quantity_kind: QuantityKind,
    pub value_q: i64,
    pub scale: Scale,
    pub unit: SiUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CbrnStructuredClaim {
    pub schema_version: SchemaVersion,
    pub profile: Profile,
    pub domain: Domain,
    pub claim_kind: ClaimKind,
    pub quantities: Vec<QuantizedValue>,
    pub envelope_id: [u8; 32],
    pub envelope_check: EnvelopeCheck,
    pub references: Vec<[u8; 32]>,
    pub etl_root: [u8; 32],
    pub envelope_manifest_hash: [u8; 32],
    pub envelope_manifest_version: u32,
    pub decision: Decision,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KoutAccounting {
    pub kout_bits: u32,
    pub capacity_bits: u32,
}

fn ceil_log2(n: usize) -> u32 {
    if n <= 1 {
        0
    } else {
        usize::BITS - (n - 1).leading_zeros()
    }
}

pub fn validate_cbrn_claim(claim: &CbrnStructuredClaim) -> Result<(), String> {
    if claim.quantities.is_empty() {
        return Err("at least one quantity is required".into());
    }
    if claim.quantities.len() > MAX_QUANTITIES {
        return Err("too many quantities".into());
    }
    if claim.references.len() > MAX_REFERENCES {
        return Err("too many references".into());
    }
    if claim.reason_codes.is_empty() {
        return Err("at least one reason_code is required".into());
    }
    if claim.reason_codes.len() > MAX_REASON_CODES {
        return Err("too many reason_codes".into());
    }

    if claim.quantities.iter().any(|q| q.value_q < 0) {
        return Err("quantity value_q must be non-negative".into());
    }

    if matches!(claim.decision, Decision::Heavy | Decision::Escalate)
        && !claim.reason_codes.iter().any(|r| {
            matches!(
                r,
                ReasonCode::AboveThreshold
                    | ReasonCode::MagnitudeEnvelopeExceeded
                    | ReasonCode::StructuralAnomalyDetected
            )
        })
    {
        return Err("heavy/escalate decision requires threshold/anomaly reason".into());
    }

    Ok(())
}

fn reject_non_integer_numbers(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Number(number) => {
            if number.is_i64() || number.is_u64() {
                Ok(())
            } else {
                Err("floating-point numbers are forbidden in CBRN-SC claims".into())
            }
        }
        serde_json::Value::Array(values) => {
            for v in values {
                reject_non_integer_numbers(v)?;
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                reject_non_integer_numbers(v)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn parse_cbrn_claim_json(bytes: &[u8]) -> Result<CbrnStructuredClaim, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid json: {e}"))?;
    reject_non_integer_numbers(&value)?;
    serde_json::from_value::<CbrnStructuredClaim>(value).map_err(|e| format!("invalid claim: {e}"))
}

pub fn canonicalize_cbrn_claim(claim: &CbrnStructuredClaim) -> Result<Vec<u8>, String> {
    validate_cbrn_claim(claim)?;

    let mut out = vec![
        claim.schema_version.discriminant(),
        claim.profile.discriminant(),
        claim.domain.discriminant(),
        claim.claim_kind.discriminant(),
        claim.quantities.len() as u8,
    ];
    for quantity in &claim.quantities {
        out.push(quantity.quantity_kind.discriminant());
        out.extend_from_slice(&quantity.value_q.to_be_bytes());
        out.push(quantity.scale.discriminant());
        out.push(quantity.unit.discriminant());
    }

    out.extend_from_slice(&claim.envelope_id);
    out.push(claim.envelope_check.discriminant());

    out.push(claim.references.len() as u8);
    for reference in &claim.references {
        out.extend_from_slice(reference);
    }

    out.extend_from_slice(&claim.etl_root);
    out.extend_from_slice(&claim.envelope_manifest_hash);
    out.extend_from_slice(&claim.envelope_manifest_version.to_be_bytes());

    out.push(claim.decision.discriminant());
    out.push(claim.reason_codes.len() as u8);
    for reason_code in &claim.reason_codes {
        out.push(reason_code.discriminant());
    }

    Ok(out)
}

pub fn kout_accounting(claim: &CbrnStructuredClaim) -> KoutAccounting {
    let quantity_bits = ceil_log2(QuantityKind::variant_count())
        + 64
        + ceil_log2(Scale::variant_count())
        + ceil_log2(SiUnit::variant_count());
    let reason_bits = ceil_log2(ReasonCode::variant_count()) * claim.reason_codes.len() as u32;
    let count_bits = ceil_log2(MAX_QUANTITIES + 1)
        + ceil_log2(MAX_REFERENCES + 1)
        + ceil_log2(MAX_REASON_CODES + 1);

    let kout_bits = ceil_log2(SchemaVersion::variant_count())
        + ceil_log2(Profile::variant_count())
        + ceil_log2(Domain::variant_count())
        + ceil_log2(ClaimKind::variant_count())
        + (claim.quantities.len() as u32 * quantity_bits)
        + 256
        + ceil_log2(EnvelopeCheck::variant_count())
        + (claim.references.len() as u32 * 256)
        + 256
        + 256
        + 32
        + ceil_log2(Decision::variant_count())
        + reason_bits
        + count_bits;

    KoutAccounting {
        kout_bits,
        capacity_bits: kout_bits,
    }
}

pub fn kout_bits(claim: &CbrnStructuredClaim) -> u32 {
    kout_accounting(claim).kout_bits
}

pub fn kout_budget_charge(claim: &CbrnStructuredClaim) -> f64 {
    kout_bits(claim) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    fn sample() -> CbrnStructuredClaim {
        CbrnStructuredClaim {
            schema_version: SchemaVersion::V1_0_0,
            profile: Profile::CbrnSc,
            domain: Domain::Cbrn,
            claim_kind: ClaimKind::Assessment,
            quantities: vec![QuantizedValue {
                quantity_kind: QuantityKind::Concentration,
                value_q: 1200,
                scale: Scale::Micro,
                unit: SiUnit::MolPerM3,
            }],
            envelope_id: [1u8; 32],
            envelope_check: EnvelopeCheck::Match,
            references: vec![[2u8; 32]],
            etl_root: [3u8; 32],
            envelope_manifest_hash: [4u8; 32],
            envelope_manifest_version: 1,
            decision: Decision::Pass,
            reason_codes: vec![ReasonCode::SensorAgreement],
        }
    }

    fn with_all_fields_populated() -> CbrnStructuredClaim {
        CbrnStructuredClaim {
            quantities: vec![
                QuantizedValue {
                    quantity_kind: QuantityKind::Concentration,
                    value_q: 1200,
                    scale: Scale::Micro,
                    unit: SiUnit::MolPerM3,
                },
                QuantizedValue {
                    quantity_kind: QuantityKind::DoseRate,
                    value_q: 42,
                    scale: Scale::Milli,
                    unit: SiUnit::GrayPerSec,
                },
            ],
            references: vec![[2u8; 32], [7u8; 32]],
            reason_codes: vec![ReasonCode::SensorAgreement, ReasonCode::AboveThreshold],
            decision: Decision::Heavy,
            envelope_manifest_version: 9,
            ..sample()
        }
    }

    #[test]
    fn canonicalization_stable_bytes() {
        let c = sample();
        assert_eq!(
            canonicalize_cbrn_claim(&c).expect("first serialization succeeds"),
            canonicalize_cbrn_claim(&c).expect("second serialization succeeds")
        );
    }

    #[test]
    fn rejects_floats() {
        let json = r#"{
          "schema_version":"v1_0_0",
          "profile":"cbrn_sc",
          "domain":"cbrn",
          "claim_kind":"assessment",
          "quantities":[{"quantity_kind":"concentration","value_q":12.34,"scale":"micro","unit":"mol_per_m3"}],
          "envelope_id":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
          "envelope_check":"match",
          "references":[],
          "etl_root":[3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3],
          "envelope_manifest_hash":[4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4],
          "envelope_manifest_version":1,
          "decision":"pass",
          "reason_codes":["sensor_agreement"]
        }"#;

        assert!(parse_cbrn_claim_json(json.as_bytes()).is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        let json = r#"{
          "schema_version":"v1_0_0",
          "profile":"cbrn_sc",
          "domain":"cbrn",
          "claim_kind":"assessment",
          "quantities":[{"quantity_kind":"concentration","value_q":123,"scale":"micro","unit":"mol_per_m3"}],
          "envelope_id":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
          "envelope_check":"match",
          "references":[],
          "etl_root":[3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3],
          "envelope_manifest_hash":[4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4],
          "envelope_manifest_version":1,
          "decision":"pass",
          "reason_codes":["sensor_agreement"],
          "unknown_field":"nope"
        }"#;

        assert!(parse_cbrn_claim_json(json.as_bytes()).is_err());
    }

    #[test]
    fn validate_rejects_empty_quantities() {
        let mut c = sample();
        c.quantities.clear();
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_rejects_too_many_quantities() {
        let mut c = sample();
        c.quantities = (0..(MAX_QUANTITIES + 1))
            .map(|_| QuantizedValue {
                quantity_kind: QuantityKind::Concentration,
                value_q: 1,
                scale: Scale::Unit,
                unit: SiUnit::MolPerM3,
            })
            .collect();
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_rejects_negative_value_q() {
        let mut c = sample();
        c.quantities[0].value_q = -1;
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_rejects_too_many_references() {
        let mut c = sample();
        c.references = vec![[9u8; 32]; MAX_REFERENCES + 1];
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_rejects_empty_reason_codes() {
        let mut c = sample();
        c.reason_codes.clear();
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_rejects_too_many_reason_codes() {
        let mut c = sample();
        c.reason_codes = vec![ReasonCode::SensorAgreement; MAX_REASON_CODES + 1];
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_heavy_requires_specific_reason_code() {
        let mut c = sample();
        c.decision = Decision::Heavy;
        c.reason_codes = vec![ReasonCode::SensorAgreement];
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn rejects_missing_required_fields() {
        let missing = json!({
            "schema_version":"v1_0_0",
            "profile":"cbrn_sc"
        });
        let bytes = serde_json::to_vec(&missing).expect("json serialization succeeds");
        assert!(parse_cbrn_claim_json(&bytes).is_err());
    }

    fn canonical_bytes_for(mutator: impl FnOnce(&mut CbrnStructuredClaim)) -> Vec<u8> {
        let mut c = with_all_fields_populated();
        mutator(&mut c);
        canonicalize_cbrn_claim(&c).expect("canonicalization succeeds")
    }

    #[test]
    fn profile_variants_encoded_distinctly() {
        assert_eq!(Profile::variant_count(), 1);
        assert_eq!(Profile::CbrnSc.discriminant(), 0);
    }

    #[test]
    fn domain_variants_encoded_distinctly() {
        assert_eq!(Domain::variant_count(), 1);
        assert_eq!(Domain::Cbrn.discriminant(), 0);
    }

    #[test]
    fn claim_kind_variants_encoded_distinctly() {
        assert_eq!(ClaimKind::variant_count(), 1);
        assert_eq!(ClaimKind::Assessment.discriminant(), 0);
    }

    #[test]
    fn quantity_kind_variants_encoded_distinctly() {
        let unique: HashSet<_> = [
            QuantityKind::Concentration,
            QuantityKind::DoseRate,
            QuantityKind::Activity,
        ]
        .iter()
        .map(QuantityKind::discriminant)
        .collect();
        assert_eq!(unique.len(), QuantityKind::variant_count());
    }

    #[test]
    fn scale_variants_encoded_distinctly() {
        let unique: HashSet<_> = [
            Scale::Unit,
            Scale::Milli,
            Scale::Micro,
            Scale::Nano,
            Scale::Pico,
            Scale::Femto,
        ]
        .iter()
        .map(Scale::discriminant)
        .collect();
        assert_eq!(unique.len(), Scale::variant_count());
    }

    #[test]
    fn unit_variants_encoded_distinctly() {
        let unique: HashSet<_> = [
            SiUnit::MolPerM3,
            SiUnit::KgPerM3,
            SiUnit::BqPerM3,
            SiUnit::JPerKg,
            SiUnit::GrayPerSec,
            SiUnit::WattPerM2,
            SiUnit::KgPerKgBody,
        ]
        .iter()
        .map(SiUnit::discriminant)
        .collect();
        assert_eq!(unique.len(), SiUnit::variant_count());
    }

    #[test]
    fn envelope_check_variants_encoded_distinctly() {
        let unique: HashSet<_> = [
            EnvelopeCheck::Match,
            EnvelopeCheck::Missing,
            EnvelopeCheck::Mismatch,
        ]
        .iter()
        .map(EnvelopeCheck::discriminant)
        .collect();
        assert_eq!(unique.len(), EnvelopeCheck::variant_count());
    }

    #[test]
    fn decision_variants_encoded_distinctly() {
        let unique: HashSet<_> = [
            Decision::Pass,
            Decision::Heavy,
            Decision::Reject,
            Decision::Escalate,
        ]
        .iter()
        .map(Decision::discriminant)
        .collect();
        assert_eq!(unique.len(), Decision::variant_count());
    }

    #[test]
    fn canonical_bytes_change_on_each_field_mutation() {
        let baseline = canonicalize_cbrn_claim(&with_all_fields_populated()).expect("baseline");
        let candidates = vec![
            canonical_bytes_for(|c| c.quantities[0].quantity_kind = QuantityKind::Activity),
            canonical_bytes_for(|c| c.quantities[0].value_q += 1),
            canonical_bytes_for(|c| c.quantities[0].scale = Scale::Nano),
            canonical_bytes_for(|c| c.quantities[0].unit = SiUnit::BqPerM3),
            canonical_bytes_for(|c| c.envelope_id[0] ^= 1),
            canonical_bytes_for(|c| c.envelope_check = EnvelopeCheck::Mismatch),
            canonical_bytes_for(|c| c.references[0][0] ^= 1),
            canonical_bytes_for(|c| c.etl_root[0] ^= 1),
            canonical_bytes_for(|c| c.envelope_manifest_hash[0] ^= 1),
            canonical_bytes_for(|c| c.envelope_manifest_version += 1),
            canonical_bytes_for(|c| c.decision = Decision::Escalate),
            canonical_bytes_for(|c| c.reason_codes[0] = ReasonCode::StructuralAnomalyDetected),
        ];
        assert!(candidates.into_iter().all(|bytes| bytes != baseline));
    }

    #[test]
    fn envelope_id_affects_canonical_bytes() {
        let baseline = canonicalize_cbrn_claim(&with_all_fields_populated()).expect("baseline");
        let changed = canonical_bytes_for(|c| c.envelope_id[31] ^= 1);
        assert_ne!(baseline, changed);
    }

    #[test]
    fn etl_root_affects_canonical_bytes() {
        let baseline = canonicalize_cbrn_claim(&with_all_fields_populated()).expect("baseline");
        let changed = canonical_bytes_for(|c| c.etl_root[31] ^= 1);
        assert_ne!(baseline, changed);
    }

    #[test]
    fn manifest_hash_affects_canonical_bytes() {
        let baseline = canonicalize_cbrn_claim(&with_all_fields_populated()).expect("baseline");
        let changed = canonical_bytes_for(|c| c.envelope_manifest_hash[31] ^= 1);
        assert_ne!(baseline, changed);
    }

    #[test]
    fn manifest_version_roundtrip() {
        let c = with_all_fields_populated();
        let v = serde_json::to_vec(&c).expect("serialize");
        let parsed = parse_cbrn_claim_json(&v).expect("parse");
        assert_eq!(
            parsed.envelope_manifest_version,
            c.envelope_manifest_version
        );
    }

    #[test]
    fn schema_version_roundtrip() {
        let c = with_all_fields_populated();
        let v = serde_json::to_vec(&c).expect("serialize");
        let parsed = parse_cbrn_claim_json(&v).expect("parse");
        assert_eq!(parsed.schema_version, c.schema_version);
    }

    #[test]
    fn kout_monotone_wrt_counts() {
        let mut few = sample();
        few.references.clear();
        few.reason_codes = vec![ReasonCode::SensorAgreement];

        let mut many = few.clone();
        many.quantities.push(QuantizedValue {
            quantity_kind: QuantityKind::Activity,
            value_q: 9,
            scale: Scale::Nano,
            unit: SiUnit::BqPerM3,
        });
        many.references.push([8u8; 32]);
        many.reason_codes.push(ReasonCode::CalibrationExpired);

        assert!(kout_accounting(&many).kout_bits >= kout_accounting(&few).kout_bits);
    }

    #[test]
    fn kout_budget_charge_matches_accounting() {
        let c = with_all_fields_populated();
        let accounting = kout_accounting(&c);
        assert_eq!(kout_budget_charge(&c), accounting.kout_bits as f64);
    }

    #[test]
    fn kout_matches_known_small_schema_case() {
        let c = sample();
        let accounting = kout_accounting(&c);
        assert_eq!(accounting.kout_bits, 1148);
        assert!(accounting.kout_bits <= accounting.capacity_bits);
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::Value;

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
            0i64..1_000_000i64,
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
        fn prop_valid_claim_roundtrip_canonicalization(claim in arb_claim()) {
            prop_assert!(validate_cbrn_claim(&claim).is_ok());
            let b1 = canonicalize_cbrn_claim(&claim).expect("first canonicalization");
            let b2 = canonicalize_cbrn_claim(&claim).expect("second canonicalization");
            prop_assert_eq!(b1.as_slice(), b2.as_slice());

            let v = serde_json::to_value(&claim).expect("serde to value");
            let encoded = serde_json::to_vec(&v).expect("value to bytes");
            let parsed = parse_cbrn_claim_json(&encoded).expect("parser accepts generated claim");
            let b3 = canonicalize_cbrn_claim(&parsed).expect("canonicalization after parse");
            prop_assert_eq!(b1.as_slice(), b3.as_slice());
        }

        #[test]
        fn prop_json_rejects_any_float_anywhere(
            claim in arb_claim(),
            depth in 0usize..4,
            idx in 0usize..16,
        ) {
            let mut value = serde_json::to_value(&claim).expect("serialize claim");
            for _ in 0..depth {
                value = Value::Array(vec![value]);
            }
            let mut root = serde_json::json!({"claim": value});
            let mut cur = &mut root["claim"];
            for _ in 0..depth {
                cur = &mut cur[0];
            }
            let quantity_len = cur["quantities"].as_array().map_or(1, Vec::len);
            let q_idx = idx % quantity_len;
            cur["quantities"][q_idx]["value_q"] = serde_json::json!(1.25);
            let bytes = serde_json::to_vec(&root["claim"]).expect("json bytes");
            prop_assert!(parse_cbrn_claim_json(&bytes).is_err());
        }

        #[test]
        fn prop_quantity_count_constraints(mut claim in arb_claim(), over in 0usize..2) {
            if over == 0 {
                claim.quantities.clear();
            } else {
                claim.quantities = vec![claim.quantities[0].clone(); MAX_QUANTITIES + 1];
            }
            prop_assert!(validate_cbrn_claim(&claim).is_err());
        }

        #[test]
        fn prop_reference_count_constraints(mut claim in arb_claim()) {
            claim.references = vec![[11u8; 32]; MAX_REFERENCES + 1];
            prop_assert!(validate_cbrn_claim(&claim).is_err());
        }

        #[test]
        fn prop_reason_code_count_constraints(mut claim in arb_claim(), over in 0usize..2) {
            if over == 0 {
                claim.reason_codes.clear();
            } else {
                claim.reason_codes = vec![ReasonCode::SensorAgreement; MAX_REASON_CODES + 1];
            }
            prop_assert!(validate_cbrn_claim(&claim).is_err());
        }

        #[test]
        fn prop_value_q_nonnegative(mut claim in arb_claim()) {
            claim.quantities[0].value_q = -1;
            prop_assert!(validate_cbrn_claim(&claim).is_err());
        }

        #[test]
        fn prop_decision_reasoncode_rules(mut claim in arb_claim()) {
            claim.decision = Decision::Escalate;
            claim.reason_codes = vec![ReasonCode::SensorAgreement];
            prop_assert!(validate_cbrn_claim(&claim).is_err());
        }

        #[test]
        fn prop_kout_monotone(claim in arb_claim(), extra_refs in 0usize..3, extra_reasons in 0usize..3) {
            let mut expanded = claim.clone();
            if expanded.quantities.len() < MAX_QUANTITIES {
                expanded.quantities.push(QuantizedValue {
                    quantity_kind: QuantityKind::Activity,
                    value_q: 7,
                    scale: Scale::Nano,
                    unit: SiUnit::BqPerM3,
                });
            }
            for _ in 0..extra_refs {
                if expanded.references.len() < MAX_REFERENCES {
                    expanded.references.push([13u8; 32]);
                }
            }
            for _ in 0..extra_reasons {
                if expanded.reason_codes.len() < MAX_REASON_CODES {
                    expanded.reason_codes.push(ReasonCode::CalibrationExpired);
                }
            }
            prop_assert!(kout_accounting(&expanded).kout_bits >= kout_accounting(&claim).kout_bits);
        }
    }
}
