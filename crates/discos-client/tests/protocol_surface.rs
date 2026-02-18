use discos_client::pb;
use tonic::transport::Channel;

#[test]
fn protocol_crate_exports_evidence_os_client_type() {
    let _client_type: Option<pb::evidence_os_client::EvidenceOsClient<Channel>> = None;
}
