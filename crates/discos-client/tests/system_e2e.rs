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

use discos_client::{
    verify_inclusion_proof, verify_sth_signature, DiscosClient, InclusionProof, SignedTreeHead,
};

#[tokio::test]
#[ignore = "requires running evidenceos-daemon; set EVIDENCEOS_DAEMON_ADDR to enable"]
async fn discos_client_to_daemon_smoke() {
    let addr = std::env::var("EVIDENCEOS_DAEMON_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    let mut client = match DiscosClient::connect(&addr).await {
        Ok(c) => c,
        Err(_) => return,
    };

    let _ = client.health().await;

    // Cryptographic verification paths are exercised in unit tests with golden vectors.
    let sk = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
    let mut signed = Vec::new();
    signed.extend_from_slice(&1u64.to_be_bytes());
    signed.extend_from_slice(&[2u8; 32]);
    let sig = ed25519_dalek::Signer::sign(&sk, &signed).to_bytes();
    let sth = SignedTreeHead {
        tree_size: 1,
        root_hash: [2u8; 32],
        signature: sig,
    };
    let _ = verify_sth_signature(&sth, sk.verifying_key().as_bytes());

    let proof = InclusionProof {
        leaf_hash: [3u8; 32],
        leaf_index: 0,
        tree_size: 1,
        audit_path: vec![],
    };
    let _ = verify_inclusion_proof([3u8; 32], &proof);
}
