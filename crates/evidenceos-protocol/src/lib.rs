#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod pb {
    tonic::include_proto!("evidenceos.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("evidenceos_descriptor");
