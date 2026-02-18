#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod pb {
    tonic::include_proto!("evidenceos.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("evidenceos_descriptor");

pub mod domains {
    pub const CAPSULE_HASH_V1: &[u8] = b"evidenceos/capsule-hash/v1";
    pub const STRUCTURED_OUTPUT_HASH_V1: &[u8] = b"evidenceos/structured-output-hash/v1";
    pub const STH_SIGNATURE_V1: &[u8] = b"evidenceos/sth-signature/v1";
    pub const REVOCATION_FEED_V1: &[u8] = b"evidenceos/revocation-feed/v1";
}
