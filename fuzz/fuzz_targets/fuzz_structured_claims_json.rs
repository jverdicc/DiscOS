#![no_main]

use discos_core::structured_claims::{
    canonicalize_cbrn_claim, kout_bits, parse_cbrn_claim_json, validate_cbrn_claim,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(claim) = parse_cbrn_claim_json(data) {
        let _ = validate_cbrn_claim(&claim);
        let _ = canonicalize_cbrn_claim(&claim);
        let _ = kout_bits(&claim);
    }
});
