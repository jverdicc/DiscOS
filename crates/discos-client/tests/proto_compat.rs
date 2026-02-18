use discos_client::pb;
use prost::Message;
use prost_types::FileDescriptorSet;
use std::collections::BTreeSet;

#[test]
fn generated_client_exposes_expected_service_methods() {
    fn _assert_methods(
        mut client: pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>,
    ) {
        let _ = client.health(pb::HealthRequest {});
        let _ = client.create_claim(pb::CreateClaimRequest::default());
        let _ = client.create_claim_v2(pb::CreateClaimV2Request::default());
        let _ = client.commit_artifacts(pb::CommitArtifactsRequest::default());
        let _ = client.freeze_gates(pb::FreezeGatesRequest::default());
        let _ = client.seal_claim(pb::SealClaimRequest::default());
        let _ = client.execute_claim(pb::ExecuteClaimRequest::default());
        let _ = client.execute_claim_v2(pb::ExecuteClaimV2Request::default());
        let _ = client.fetch_capsule(pb::FetchCapsuleRequest::default());
        let _ = client.get_signed_tree_head(pb::GetSignedTreeHeadRequest::default());
        let _ = client.get_inclusion_proof(pb::GetInclusionProofRequest::default());
        let _ = client.get_consistency_proof(pb::GetConsistencyProofRequest::default());
        let _ = client.revoke_claim(pb::RevokeClaimRequest::default());
        let _ = client.watch_revocations(pb::WatchRevocationsRequest::default());
    }

    let _ = _assert_methods;
}

#[test]
fn descriptor_contains_required_v2_rpcs() {
    let descriptors = FileDescriptorSet::decode(evidenceos_protocol::FILE_DESCRIPTOR_SET)
        .expect("decode embedded file descriptor set");

    let mut methods = BTreeSet::new();
    for file in descriptors.file {
        let package = file.package.unwrap_or_default();
        for service in file.service {
            let service_name = service.name.unwrap_or_default();
            for method in service.method {
                let method_name = method.name.unwrap_or_default();
                methods.insert(format!("{}.{}.{method_name}", package, service_name));
            }
        }
    }

    let required = [
        "evidenceos.v1.EvidenceOS.CreateClaimV2",
        "evidenceos.v1.EvidenceOS.CommitArtifacts",
        "evidenceos.v1.EvidenceOS.ExecuteClaimV2",
        "evidenceos.v1.EvidenceOS.FetchCapsule",
        "evidenceos.v1.EvidenceOS.GetSignedTreeHead",
        "evidenceos.v1.EvidenceOS.WatchRevocations",
    ];

    for rpc in required {
        assert!(methods.contains(rpc), "missing required rpc {rpc}");
    }

    let _has_get_public_key = methods.contains("evidenceos.v1.EvidenceOS.GetPublicKey");
}
