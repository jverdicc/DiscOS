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

use discos_client::pb;
use prost::Message;
use prost_types::FileDescriptorSet;
use std::collections::BTreeSet;

#[test]
fn generated_client_exposes_expected_service_methods() {
    fn _assert_methods(
        mut client: pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>,
    ) {
        std::mem::drop(client.health(pb::HealthRequest {}));
        std::mem::drop(client.create_claim(pb::CreateClaimRequest::default()));
        std::mem::drop(client.create_claim_v2(pb::CreateClaimV2Request::default()));
        std::mem::drop(client.commit_artifacts(pb::CommitArtifactsRequest::default()));
        std::mem::drop(client.freeze_gates(pb::FreezeGatesRequest::default()));
        std::mem::drop(client.seal_claim(pb::SealClaimRequest::default()));
        std::mem::drop(client.execute_claim(pb::ExecuteClaimRequest::default()));
        std::mem::drop(client.execute_claim_v2(pb::ExecuteClaimV2Request::default()));
        std::mem::drop(client.fetch_capsule(pb::FetchCapsuleRequest::default()));
        std::mem::drop(client.get_signed_tree_head(pb::GetSignedTreeHeadRequest::default()));
        std::mem::drop(client.get_inclusion_proof(pb::GetInclusionProofRequest::default()));
        std::mem::drop(client.get_consistency_proof(pb::GetConsistencyProofRequest::default()));
        std::mem::drop(client.revoke_claim(pb::RevokeClaimRequest::default()));
        std::mem::drop(client.watch_revocations(pb::WatchRevocationsRequest::default()));
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

#[test]
fn claim_id_fields_are_bytes_on_v2_surface() {
    let descriptors = FileDescriptorSet::decode(evidenceos_protocol::FILE_DESCRIPTOR_SET)
        .expect("decode embedded file descriptor set");

    let mut field_types = std::collections::BTreeMap::new();
    for file in descriptors.file {
        for message in file.message_type {
            let message_name = message.name.unwrap_or_default();
            for field in message.field {
                if field.name.as_deref() == Some("claim_id") {
                    field_types.insert(message_name.clone(), field.r#type);
                }
            }
        }
    }

    let expected_bytes = Some(prost_types::field_descriptor_proto::Type::Bytes as i32);
    for message in [
        "CreateClaimResponse",
        "CreateClaimV2Response",
        "CommitArtifactsRequest",
        "FreezeGatesRequest",
        "SealClaimRequest",
        "ExecuteClaimV2Request",
        "FetchCapsuleRequest",
        "RevokeClaimRequest",
    ] {
        assert_eq!(
            field_types.get(message).copied().flatten(),
            expected_bytes,
            "{message}.claim_id must be bytes",
        );
    }
}
