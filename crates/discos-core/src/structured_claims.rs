use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
pub enum Analyte {
    Nh3,
    Cl2,
    Cs137,
    Vx,
    Gb,
    I131,
    Co60,
    Anthrax,
    Unknown,
}

impl Analyte {
    pub const fn variant_count() -> usize {
        9
    }

    pub const fn discriminant(&self) -> u8 {
        match self {
            Self::Nh3 => 0,
            Self::Cl2 => 1,
            Self::Cs137 => 2,
            Self::Vx => 3,
            Self::Gb => 4,
            Self::I131 => 5,
            Self::Co60 => 6,
            Self::Anthrax => 7,
            Self::Unknown => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuantizedValue {
    pub value_q: i64,
    pub scale: Scale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbrnStructuredClaim {
    pub schema_id: String,
    pub analyte: Analyte,
    pub concentration: QuantizedValue,
    pub unit: SiUnit,
    pub confidence_pct_x100: u16,
    pub decision: Decision,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn validate_cbrn_claim(claim: &CbrnStructuredClaim) -> Result<(), String> {
    if claim.schema_id != "cbrn-sc.v1" {
        return Err("unsupported schema_id".into());
    }
    if claim.confidence_pct_x100 > 10_000 {
        return Err("confidence out of range".into());
    }
    if claim.reason_codes.is_empty() {
        return Err("at least one reason_code is required".into());
    }
    if claim.reason_codes.len() > 8 {
        return Err("at most 8 reason_codes are allowed".into());
    }
    if claim.concentration.value_q < 0 {
        return Err("value_q must be non-negative".into());
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

pub fn canonicalize_cbrn_claim(claim: &CbrnStructuredClaim) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let schema = claim.schema_id.as_bytes();
    out.extend_from_slice(&(schema.len() as u32).to_be_bytes());
    out.extend_from_slice(schema);
    out.push(claim.analyte.discriminant());
    out.extend_from_slice(&claim.concentration.value_q.to_be_bytes());
    out.push(claim.concentration.scale.discriminant());
    out.push(claim.unit.discriminant());
    out.extend_from_slice(&claim.confidence_pct_x100.to_be_bytes());
    out.push(claim.decision.discriminant());
    out.push(claim.reason_codes.len() as u8);
    for rc in &claim.reason_codes {
        out.push(rc.discriminant());
    }
    Ok(out)
}

pub fn kout_bits(claim: &CbrnStructuredClaim) -> u32 {
    let analyte_bits = f64::log2(Analyte::variant_count() as f64).ceil() as u32;
    let scale_bits = f64::log2(Scale::variant_count() as f64).ceil() as u32;
    let unit_bits = f64::log2(SiUnit::variant_count() as f64).ceil() as u32;
    let decision_bits = f64::log2(Decision::variant_count() as f64).ceil() as u32;
    let confidence_bits = 14;
    let reason_bits = (f64::log2(ReasonCode::variant_count() as f64).ceil() as u32)
        * claim.reason_codes.len() as u32;

    analyte_bits + scale_bits + unit_bits + decision_bits + confidence_bits + reason_bits
}

pub fn kout_budget_charge(claim: &CbrnStructuredClaim) -> f64 {
    kout_bits(claim) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CbrnStructuredClaim {
        CbrnStructuredClaim {
            schema_id: "cbrn-sc.v1".into(),
            analyte: Analyte::Nh3,
            concentration: QuantizedValue {
                value_q: 1200,
                scale: Scale::Micro,
            },
            unit: SiUnit::MolPerM3,
            confidence_pct_x100: 9123,
            decision: Decision::Pass,
            reason_codes: vec![ReasonCode::SensorAgreement],
        }
    }

    #[test]
    fn kout_bits_increases_with_more_reason_codes() {
        let mut c = sample();
        let a = kout_bits(&c);
        c.reason_codes = vec![
            ReasonCode::SensorAgreement,
            ReasonCode::AboveThreshold,
            ReasonCode::CalibrationExpired,
            ReasonCode::LineageTainted,
        ];
        assert!(kout_bits(&c) > a);
    }

    #[test]
    fn canonicalize_stable() {
        let c = sample();
        assert_eq!(
            canonicalize_cbrn_claim(&c).expect("first serialization succeeds"),
            canonicalize_cbrn_claim(&c).expect("second serialization succeeds")
        );
    }

    #[test]
    fn validate_confidence_10001_err() {
        let mut c = sample();
        c.confidence_pct_x100 = 10_001;
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn validate_negative_value_q_err() {
        let mut c = sample();
        c.concentration.value_q = -1;
        assert!(validate_cbrn_claim(&c).is_err());
    }
    #[test]
    fn heavy_requires_threshold_reason() {
        let mut c = sample();
        c.decision = Decision::Heavy;
        c.reason_codes = vec![ReasonCode::SensorAgreement];
        assert!(validate_cbrn_claim(&c).is_err());

        c.reason_codes.push(ReasonCode::AboveThreshold);
        assert!(validate_cbrn_claim(&c).is_ok());
    }

    #[test]
    fn rejects_more_than_eight_reason_codes() {
        let mut c = sample();
        c.reason_codes = vec![
            ReasonCode::SensorAgreement,
            ReasonCode::AboveThreshold,
            ReasonCode::BelowThreshold,
            ReasonCode::IncompleteInputs,
            ReasonCode::MagnitudeEnvelopeExceeded,
            ReasonCode::CalibrationExpired,
            ReasonCode::LineageTainted,
            ReasonCode::StructuralAnomalyDetected,
            ReasonCode::AboveThreshold,
        ];
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn discriminants_are_stable() {
        assert_eq!(Scale::Unit.discriminant(), 0);
        assert_eq!(SiUnit::MolPerM3.discriminant(), 0);
        assert_eq!(Decision::Escalate.discriminant(), 3);
        assert_eq!(Analyte::Unknown.discriminant(), 8);
        assert_eq!(ReasonCode::StructuralAnomalyDetected.discriminant(), 7);
    }

    #[test]
    fn canonical_bytes_change_on_any_field_change() {
        let c = sample();
        let base = canonicalize_cbrn_claim(&c).unwrap_or_default();

        let mut changed = c.clone();
        changed.analyte = Analyte::Cl2;
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());

        changed = c.clone();
        changed.concentration.value_q += 1;
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());

        changed = c.clone();
        changed.concentration.scale = Scale::Nano;
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());

        changed = c.clone();
        changed.unit = SiUnit::GrayPerSec;
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());

        changed = c.clone();
        changed.confidence_pct_x100 -= 1;
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());

        changed = c.clone();
        changed.decision = Decision::Reject;
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());

        changed = c;
        changed.reason_codes = vec![ReasonCode::CalibrationExpired];
        assert_ne!(base, canonicalize_cbrn_claim(&changed).unwrap_or_default());
    }
}
