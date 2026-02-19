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

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod domains {
    pub const CAPSULE_HASH_V1: &[u8] = b"evidenceos/capsule-hash/v1";
    pub const STH_SIGNATURE_V1: &[u8] = b"evidenceos/sth-signature/v1";
    pub const REVOCATION_FEED_V1: &[u8] = b"evidenceos/revocation-feed/v1";
}

pub mod pb {
    tonic::include_proto!("evidenceos.v1");
}

pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/evidenceos_descriptor.bin"));
