use evidenceos_auth_protocol::{sign_hmac_sha256, signing_material, HMAC_SHA256_TEST_VECTORS};

#[test]
fn hmac_signatures_match_shared_auth_protocol_vectors() {
    for vector in HMAC_SHA256_TEST_VECTORS {
        let material = signing_material(vector.request_id, vector.path, vector.timestamp);
        let signature = sign_hmac_sha256(vector.secret, &material);
        assert_eq!(hex::encode(signature), vector.expected_signature_hex);
    }
}
