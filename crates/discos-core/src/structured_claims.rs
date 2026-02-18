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

    let mut out = Vec::new();
    out.push(claim.schema_version.discriminant());
    out.push(claim.profile.discriminant());
    out.push(claim.domain.discriminant());
    out.push(claim.claim_kind.discriminant());

    out.push(claim.quantities.len() as u8);
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
    fn kout_matches_known_small_schema_case() {
        let c = sample();
        let accounting = kout_accounting(&c);
        assert_eq!(accounting.kout_bits, 1148);
        assert!(accounting.kout_bits <= accounting.capacity_bits);
    }
}
