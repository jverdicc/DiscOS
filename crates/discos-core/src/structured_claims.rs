use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Scale {
    Unit,
    Milli,
    Micro,
    Nano,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SiUnit {
    MolPerM3,
    KgPerM3,
    BqPerM3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Pass,
    Heavy,
    Reject,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Analyte {
    Nh3,
    Cl2,
    Cs137,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasonCode {
    SensorAgreement,
    AboveThreshold,
    BelowThreshold,
    IncompleteInputs,
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
    Ok(())
}

pub fn canonicalize_cbrn_claim(claim: &CbrnStructuredClaim) -> Result<String, serde_json::Error> {
    serde_json::to_string(claim)
}

pub fn kout_bits(_claim: &CbrnStructuredClaim) -> u32 {
    // analyte(2) + scale(2) + unit(2) + decision(2) + confidence(14) + reason(3 bits/item, max 4)
    2 + 2 + 2 + 2 + 14 + 12
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
    fn rejects_invalid_fields() {
        let mut c = sample();
        c.schema_id = "bad".into();
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn canonicalization_stable() {
        let c = sample();
        let a = canonicalize_cbrn_claim(&c).expect("serialize");
        let b = canonicalize_cbrn_claim(&c).expect("serialize");
        assert_eq!(a, b);
    }

    #[test]
    fn kout_stable_fixture() {
        let c = sample();
        assert_eq!(kout_bits(&c), 34);
    }
}
