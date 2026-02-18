use discos_client::{
    merkle_leaf_hash, pb, sha256, verify_inclusion, verify_sth_signature, DiscosClient,
    InclusionProof, SignedTreeHead,
};
use prost::Message;
use serde_json::Value;

#[derive(Clone, PartialEq, Message)]
struct GetPublicKeyRequest {}

#[derive(Clone, PartialEq, Message)]
struct GetPublicKeyResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub public_key: Vec<u8>,
}

trait ClaimIdAsBytes {
    fn as_bytes_slice(&self) -> &[u8];
}

impl ClaimIdAsBytes for String {
    fn as_bytes_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ClaimIdAsBytes for Vec<u8> {
    fn as_bytes_slice(&self) -> &[u8] {
        self.as_slice()
    }
}

fn claim_id_as_bytes<T: ClaimIdAsBytes>(claim_id: &T) -> &[u8] {
    claim_id.as_bytes_slice()
}

fn parse_hash_bytes(raw: &Value) -> Option<Vec<u8>> {
    if let Some(v) = raw.as_str() {
        if let Ok(bytes) = hex::decode(v) {
            return Some(bytes);
        }
        if let Ok(bytes) = base64::decode(v) {
            return Some(bytes);
        }
        return None;
    }

    raw.as_array().and_then(|parts| {
        parts
            .iter()
            .map(|p| p.as_u64().and_then(|v| u8::try_from(v).ok()))
            .collect()
    })
}

#[tokio::test]
#[ignore = "requires running evidenceos-daemon with secure v2 endpoints"]
async fn claim_lifecycle_v2_against_daemon() {
    let addr = std::env::var("EVIDENCEOS_DAEMON_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    let claim_name = "discos-v2-e2e".to_string();

    let mut client = DiscosClient::connect(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to connect to daemon at {addr}: {e}"));

    let create = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name,
            metadata: Some(pb::ClaimMetadata {
                lane: "lane-a".to_string(),
                alpha_micros: 10,
                epoch_config_ref: "epoch-default".to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignals {
                semantic_hash: vec![],
                phys_hir_signature_hash: vec![7u8; 32],
                dependency_merkle_root: vec![],
            }),
            holdout_ref: "demo_labels.json".to_string(),
            epoch_size: 32,
            oracle_num_symbols: 8,
            access_credit: 10,
        })
        .await
        .unwrap_or_else(|e| panic!("create_claim_v2 failed: {e}"));

    let commit = client
        .commit_artifacts(pb::CommitArtifactsRequest {
            claim_id: create.claim_id.clone(),
            wasm_module: vec![0, 97, 115, 109, 1, 0, 0, 0],
            wasm_hash: vec![0u8; 32],
            manifests: vec![],
        })
        .await
        .unwrap_or_else(|e| panic!("commit_artifacts failed: {e}"));
    assert!(commit.accepted, "commit_artifacts returned accepted=false");

    let frozen = client
        .freeze_gates(pb::FreezeGatesRequest {
            claim_id: create.claim_id.clone(),
        })
        .await
        .unwrap_or_else(|e| panic!("freeze_gates failed: {e}"));
    assert!(frozen.frozen, "freeze_gates returned frozen=false");

    let sealed = client
        .seal_claim(pb::SealClaimRequest {
            claim_id: create.claim_id.clone(),
        })
        .await
        .unwrap_or_else(|e| panic!("seal_claim failed: {e}"));
    assert!(sealed.sealed, "seal_claim returned sealed=false");

    let exec = client
        .execute_claim_v2(pb::ExecuteClaimV2Request {
            claim_id: create.claim_id.clone(),
        })
        .await
        .unwrap_or_else(|e| panic!("execute_claim_v2 failed: {e}"));
    assert!(exec.certified, "execute_claim_v2 returned certified=false");
    assert!(
        !exec.canonical_output.is_empty(),
        "execute_claim_v2 canonical_output is empty"
    );

    let capsule = client
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: create.claim_id.clone(),
        })
        .await
        .unwrap_or_else(|e| panic!("fetch_capsule failed: {e}"));
    assert!(!capsule.capsule.is_empty(), "fetched capsule is empty");

    let capsule_hash = merkle_leaf_hash(&capsule.capsule);
    let inclusion = capsule
        .inclusion
        .clone()
        .unwrap_or_else(|| panic!("fetch_capsule did not include inclusion proof"));
    let inclusion_leaf_hash: [u8; 32] = inclusion
        .leaf_hash
        .clone()
        .try_into()
        .unwrap_or_else(|_| panic!("inclusion leaf_hash must be 32 bytes"));
    assert_eq!(
        capsule_hash, inclusion_leaf_hash,
        "capsule hash must match inclusion leaf hash (domain-separated)"
    );

    let proof = InclusionProof {
        leaf_hash: inclusion_leaf_hash,
        leaf_index: inclusion.leaf_index,
        tree_size: inclusion.tree_size,
        audit_path: inclusion
            .audit_path
            .into_iter()
            .map(|x| {
                x.try_into()
                    .unwrap_or_else(|_| panic!("all inclusion audit path nodes must be 32 bytes"))
            })
            .collect(),
    };

    let _ = canonical_output_matches_capsule(
        &exec.canonical_output,
        &capsule.capsule,
        claim_id_as_bytes(&create.claim_id),
        &create.topic_id,
    );

    if let Some(inclusion) = capsule.inclusion {
        if let Ok(leaf_hash) = inclusion.leaf_hash.clone().try_into() {
            let proof = InclusionProof {
                leaf_hash,
                leaf_index: inclusion.leaf_index,
                tree_size: inclusion.tree_size,
                audit_path: inclusion
                    .audit_path
                    .into_iter()
                    .filter_map(|x| x.try_into().ok())
                    .collect(),
            };
            let _ = verify_inclusion_proof(leaf_hash, &proof);
            let _ = verify_inclusion(
                capsule
                    .etl_root_hash
                    .clone()
                    .try_into()
                    .unwrap_or([0u8; 32]),
                &proof,
            );
        }
    }
    let etl_root_hash: [u8; 32] = capsule
        .etl_root_hash
        .clone()
        .try_into()
        .unwrap_or_else(|_| panic!("fetch_capsule etl_root_hash must be 32 bytes"));
    assert!(
        verify_inclusion(etl_root_hash, &proof),
        "inclusion proof must verify against capsule etl_root_hash"
    );

    let capsule_json: Value = serde_json::from_slice(&capsule.capsule)
        .unwrap_or_else(|e| panic!("capsule payload is not valid JSON: {e}"));
    let structured_output_hash = capsule_json
        .get("structured_output_hash")
        .unwrap_or_else(|| panic!("capsule JSON missing `structured_output_hash` field"));
    let structured_output_hash = parse_hash_bytes(structured_output_hash)
        .unwrap_or_else(|| panic!("capsule structured_output_hash is not parseable"));
    assert_eq!(
        structured_output_hash,
        sha256(&exec.canonical_output).to_vec(),
        "capsule structured_output_hash must equal sha256(execute canonical_output)"
    );

    let sth = client
        .get_signed_tree_head(pb::GetSignedTreeHeadRequest {})
        .await
        .unwrap_or_else(|e| panic!("get_signed_tree_head failed: {e}"));
    let sth = sth
        .sth
        .unwrap_or_else(|| panic!("get_signed_tree_head returned empty sth"));
    let sth_root_hash: [u8; 32] = sth
        .root_hash
        .try_into()
        .unwrap_or_else(|_| panic!("sth root_hash must be 32 bytes"));
    let sth_signature: [u8; 64] = sth
        .signature
        .try_into()
        .unwrap_or_else(|_| panic!("sth signature must be 64 bytes"));
    let signed_tree_head = SignedTreeHead {
        tree_size: sth.tree_size,
        root_hash: sth_root_hash,
        signature: sth_signature,
    };
    assert_eq!(
        etl_root_hash, signed_tree_head.root_hash,
        "inclusion proof root must match latest signed tree head root"
    );

    let mut grpc = tonic::client::Grpc::new(
        tonic::transport::Channel::from_shared(addr.clone())
            .unwrap_or_else(|e| panic!("invalid daemon address {addr}: {e}"))
            .connect()
            .await
            .unwrap_or_else(|e| panic!("failed to reconnect for GetPublicKey at {addr}: {e}")),
    );
    let get_public_key_resp: GetPublicKeyResponse = grpc
        .unary(
            tonic::Request::new(GetPublicKeyRequest {}),
            http::uri::PathAndQuery::from_static("/evidenceos.v1.EvidenceOS/GetPublicKey"),
            tonic::codec::ProstCodec::default(),
        )
        .await
        .unwrap_or_else(|e| panic!("GetPublicKey RPC failed: {e}"))
        .into_inner();
    verify_sth_signature(&signed_tree_head, &get_public_key_resp.public_key)
        .unwrap_or_else(|e| panic!("signed tree head signature verification failed: {e}"));

    let revoke = client
        .revoke_claim(pb::RevokeClaimRequest {
            claim_id: create.claim_id,
            reason_code: "test_revoke".to_string(),
        })
        .await
        .unwrap_or_else(|e| panic!("revoke_claim failed: {e}"));
    assert!(revoke.revoked, "revoke_claim returned revoked=false");
}
