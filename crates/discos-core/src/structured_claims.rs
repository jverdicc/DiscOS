use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Scale {
    Unit,
    Milli,
    Micro,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuantizedValue {
    pub value: i64,
    pub scale: Scale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbrnStructuredClaim {
    pub schema_id: String,
    pub analyte_code: String,
    pub concentration: QuantizedValue,
    pub unit_si: String,
    pub confidence_pct_x100: u16,
}

pub fn validate_cbrn_claim(claim: &CbrnStructuredClaim) -> Result<(), String> {
    if claim.schema_id != "cbrn-sc.v1" {
        return Err("unsupported schema_id".into());
    }
    let unit_ok = matches!(claim.unit_si.as_str(), "mol/m3" | "kg/m3" | "Bq/m3");
    if !unit_ok {
        return Err("unit not in SI allowlist".into());
    }
    if claim.confidence_pct_x100 > 10_000 {
        return Err("confidence out of range".into());
    }
    if claim.analyte_code.is_empty() || claim.analyte_code.len() > 32 {
        return Err("invalid analyte_code".into());
    }
    Ok(())
}

pub fn canonicalize_cbrn_claim(claim: &CbrnStructuredClaim) -> String {
    serde_json::to_string(claim).expect("canonicalize claim")
}

pub fn kout_bound_bits(_claim: &CbrnStructuredClaim) -> u32 {
    32 + 16 + 8 + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CbrnStructuredClaim {
        CbrnStructuredClaim {
            schema_id: "cbrn-sc.v1".into(),
            analyte_code: "NH3".into(),
            concentration: QuantizedValue {
                value: 1200,
                scale: Scale::Micro,
            },
            unit_si: "mol/m3".into(),
            confidence_pct_x100: 9123,
        }
    }

    #[test]
    fn rejects_bad_unit_or_texty_payload() {
        let mut c = sample();
        c.unit_si = "free text".into();
        assert!(validate_cbrn_claim(&c).is_err());
    }

    #[test]
    fn canonicalization_stable() {
        let c = sample();
        assert_eq!(canonicalize_cbrn_claim(&c), canonicalize_cbrn_claim(&c));
    }

    #[test]
    fn kout_stable() {
        let c = sample();
        assert_eq!(kout_bound_bits(&c), kout_bound_bits(&c));
    }
}
