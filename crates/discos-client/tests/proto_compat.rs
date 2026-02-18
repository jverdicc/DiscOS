use discos_client::pb;

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
