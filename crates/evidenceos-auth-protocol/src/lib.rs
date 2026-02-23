use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const SIGNATURE_PREFIX: &str = "sha256=";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthHeaders {
    pub request_id: String,
    pub timestamp: Option<String>,
    pub signature: String,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HmacVector {
    pub request_id: &'static str,
    pub path: &'static str,
    pub timestamp: Option<&'static str>,
    pub secret: &'static [u8],
    pub expected_signature_hex: &'static str,
}

pub const HMAC_SHA256_TEST_VECTORS: &[HmacVector] = &[
    HmacVector {
        request_id: "req-1",
        path: "/evidenceos.v2.EvidenceOS/Health",
        timestamp: None,
        secret: b"discos-shared-secret",
        expected_signature_hex: "bcbd74399d155e8abe362c7f3cb83cf1b50fa894f243a8b4f63ecbe92e4b9636",
    },
    HmacVector {
        request_id: "req-1",
        path: "/evidenceos.v2.EvidenceOS/CreateClaimV2",
        timestamp: Some("1700000000"),
        secret: b"discos-shared-secret",
        expected_signature_hex: "6eb7d9afc53e140b05ce6fb79933b8c95a8bc4b28790f8ec6ec38e03b5a9978b",
    },
];

pub fn signing_material(request_id: &str, path: &str, timestamp: Option<&str>) -> String {
    match timestamp {
        Some(ts) => format!("{request_id}:{path}:{ts}"),
        None => format!("{request_id}:{path}"),
    }
}

pub fn sign_hmac_sha256(secret: &[u8], signing_material: &str) -> [u8; 32] {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret).expect("hmac supports variable key lengths");
    mac.update(signing_material.as_bytes());
    mac.finalize().into_bytes().into()
}

pub fn signature_header_value(signature_bytes: &[u8]) -> String {
    format!("{SIGNATURE_PREFIX}{}", hex::encode(signature_bytes))
}

pub fn build_hmac_headers(
    request_id: &str,
    path: &str,
    timestamp: Option<&str>,
    secret: &[u8],
    key_id: Option<&str>,
) -> AuthHeaders {
    let material = signing_material(request_id, path, timestamp);
    let signature = sign_hmac_sha256(secret, &material);
    AuthHeaders {
        request_id: request_id.to_string(),
        timestamp: timestamp.map(ToOwned::to_owned),
        signature: signature_header_value(&signature),
        key_id: key_id.map(ToOwned::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors_match() {
        for vector in HMAC_SHA256_TEST_VECTORS {
            let material = signing_material(vector.request_id, vector.path, vector.timestamp);
            let signature = sign_hmac_sha256(vector.secret, &material);
            assert_eq!(hex::encode(signature), vector.expected_signature_hex);
        }
    }
}
