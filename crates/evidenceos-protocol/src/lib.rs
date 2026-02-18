#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod pb {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HealthRequest {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HealthResponse {
        #[prost(string, tag = "1")]
        pub status: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ClaimMetadata {
        #[prost(string, tag = "1")]
        pub lane: ::prost::alloc::string::String,
        #[prost(uint32, tag = "2")]
        pub alpha_micros: u32,
        #[prost(string, tag = "3")]
        pub epoch_config_ref: ::prost::alloc::string::String,
        #[prost(string, tag = "4")]
        pub output_schema_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct TopicSignals {
        #[prost(bytes = "vec", tag = "1")]
        pub semantic_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "2")]
        pub phys_hir_signature_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "3")]
        pub dependency_merkle_root: ::prost::alloc::vec::Vec<u8>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CreateClaimRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub metadata: ::core::option::Option<ClaimMetadata>,
        #[prost(message, optional, tag = "3")]
        pub signals: ::core::option::Option<TopicSignals>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CreateClaimResponse {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
        #[prost(bytes = "vec", tag = "2")]
        pub topic_id: ::prost::alloc::vec::Vec<u8>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ArtifactManifest {
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        #[prost(bytes = "vec", tag = "2")]
        pub canonical_bytes: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "3")]
        pub digest: ::prost::alloc::vec::Vec<u8>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CommitArtifactsRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
        #[prost(bytes = "vec", tag = "2")]
        pub wasm_module: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "3")]
        pub wasm_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(message, repeated, tag = "4")]
        pub manifests: ::prost::alloc::vec::Vec<ArtifactManifest>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CommitArtifactsResponse {
        #[prost(bool, tag = "1")]
        pub accepted: bool,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FreezeGatesRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FreezeGatesResponse {
        #[prost(bool, tag = "1")]
        pub frozen: bool,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SealClaimRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SealClaimResponse {
        #[prost(bool, tag = "1")]
        pub sealed: bool,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExecuteClaimRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExecuteClaimResponse {
        #[prost(bool, tag = "1")]
        pub executed: bool,
        #[prost(string, tag = "2")]
        pub execution_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FetchCapsuleRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MerkleInclusionProof {
        #[prost(bytes = "vec", tag = "1")]
        pub leaf_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(uint64, tag = "2")]
        pub leaf_index: u64,
        #[prost(uint64, tag = "3")]
        pub tree_size: u64,
        #[prost(bytes = "vec", repeated, tag = "4")]
        pub audit_path: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MerkleConsistencyProof {
        #[prost(uint64, tag = "1")]
        pub old_tree_size: u64,
        #[prost(uint64, tag = "2")]
        pub new_tree_size: u64,
        #[prost(bytes = "vec", repeated, tag = "3")]
        pub path: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FetchCapsuleResponse {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
        #[prost(bytes = "vec", tag = "2")]
        pub capsule: ::prost::alloc::vec::Vec<u8>,
        #[prost(uint64, tag = "3")]
        pub etl_index: u64,
        #[prost(uint64, tag = "4")]
        pub etl_tree_size: u64,
        #[prost(bytes = "vec", tag = "5")]
        pub etl_root_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "6")]
        pub sth_signature: ::prost::alloc::vec::Vec<u8>,
        #[prost(message, optional, tag = "7")]
        pub inclusion: ::core::option::Option<MerkleInclusionProof>,
        #[prost(message, optional, tag = "8")]
        pub consistency: ::core::option::Option<MerkleConsistencyProof>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SignedTreeHead {
        #[prost(uint64, tag = "1")]
        pub tree_size: u64,
        #[prost(bytes = "vec", tag = "2")]
        pub root_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes = "vec", tag = "3")]
        pub signature: ::prost::alloc::vec::Vec<u8>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetSignedTreeHeadRequest {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetSignedTreeHeadResponse {
        #[prost(message, optional, tag = "1")]
        pub sth: ::core::option::Option<SignedTreeHead>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetInclusionProofRequest {
        #[prost(bytes = "vec", tag = "1")]
        pub leaf_hash: ::prost::alloc::vec::Vec<u8>,
        #[prost(uint64, tag = "2")]
        pub leaf_index: u64,
        #[prost(uint64, tag = "3")]
        pub tree_size: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetInclusionProofResponse {
        #[prost(message, optional, tag = "1")]
        pub proof: ::core::option::Option<MerkleInclusionProof>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetConsistencyProofRequest {
        #[prost(uint64, tag = "1")]
        pub old_tree_size: u64,
        #[prost(uint64, tag = "2")]
        pub new_tree_size: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetConsistencyProofResponse {
        #[prost(message, optional, tag = "1")]
        pub proof: ::core::option::Option<MerkleConsistencyProof>,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RevokeClaimRequest {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
        #[prost(string, tag = "2")]
        pub reason_code: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RevokeClaimResponse {
        #[prost(bool, tag = "1")]
        pub revoked: bool,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct WatchRevocationsRequest {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RevocationEvent {
        #[prost(string, tag = "1")]
        pub claim_id: ::prost::alloc::string::String,
        #[prost(string, tag = "2")]
        pub reason_code: ::prost::alloc::string::String,
        #[prost(uint64, tag = "3")]
        pub logical_epoch: u64,
    }

    pub mod evidence_os_client {
        use tonic::codegen::*;

        #[derive(Debug, Clone)]
        pub struct EvidenceOsClient<T> {
            inner: tonic::client::Grpc<T>,
        }

        impl EvidenceOsClient<tonic::transport::Channel> {
            pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
            where
                D: TryInto<tonic::transport::Endpoint>,
                D::Error: Into<StdError>,
            {
                let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
                Ok(Self::new(conn))
            }
        }

        impl<T> EvidenceOsClient<T>
        where
            T: tonic::client::GrpcService<tonic::body::BoxBody>,
            T::Error: Into<StdError>,
            T::ResponseBody: Body<Data = bytes::Bytes> + Send + 'static,
            <T::ResponseBody as Body>::Error: Into<StdError> + Send,
        {
            pub fn new(inner: T) -> Self {
                Self {
                    inner: tonic::client::Grpc::new(inner),
                }
            }

            pub async fn health(
                &mut self,
                request: impl tonic::IntoRequest<super::HealthRequest>,
            ) -> Result<tonic::Response<super::HealthResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static("/evidenceos.v1.EvidenceOS/Health"),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn create_claim(
                &mut self,
                request: impl tonic::IntoRequest<super::CreateClaimRequest>,
            ) -> Result<tonic::Response<super::CreateClaimResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/CreateClaim",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn commit_artifacts(
                &mut self,
                request: impl tonic::IntoRequest<super::CommitArtifactsRequest>,
            ) -> Result<tonic::Response<super::CommitArtifactsResponse>, tonic::Status>
            {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/CommitArtifacts",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn freeze_gates(
                &mut self,
                request: impl tonic::IntoRequest<super::FreezeGatesRequest>,
            ) -> Result<tonic::Response<super::FreezeGatesResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/FreezeGates",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn seal_claim(
                &mut self,
                request: impl tonic::IntoRequest<super::SealClaimRequest>,
            ) -> Result<tonic::Response<super::SealClaimResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static("/evidenceos.v1.EvidenceOS/SealClaim"),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn execute_claim(
                &mut self,
                request: impl tonic::IntoRequest<super::ExecuteClaimRequest>,
            ) -> Result<tonic::Response<super::ExecuteClaimResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/ExecuteClaim",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn fetch_capsule(
                &mut self,
                request: impl tonic::IntoRequest<super::FetchCapsuleRequest>,
            ) -> Result<tonic::Response<super::FetchCapsuleResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/FetchCapsule",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn get_signed_tree_head(
                &mut self,
                request: impl tonic::IntoRequest<super::GetSignedTreeHeadRequest>,
            ) -> Result<tonic::Response<super::GetSignedTreeHeadResponse>, tonic::Status>
            {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/GetSignedTreeHead",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn get_inclusion_proof(
                &mut self,
                request: impl tonic::IntoRequest<super::GetInclusionProofRequest>,
            ) -> Result<tonic::Response<super::GetInclusionProofResponse>, tonic::Status>
            {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/GetInclusionProof",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn get_consistency_proof(
                &mut self,
                request: impl tonic::IntoRequest<super::GetConsistencyProofRequest>,
            ) -> Result<tonic::Response<super::GetConsistencyProofResponse>, tonic::Status>
            {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/GetConsistencyProof",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn revoke_claim(
                &mut self,
                request: impl tonic::IntoRequest<super::RevokeClaimRequest>,
            ) -> Result<tonic::Response<super::RevokeClaimResponse>, tonic::Status> {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .unary(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/RevokeClaim",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }

            pub async fn watch_revocations(
                &mut self,
                request: impl tonic::IntoRequest<super::WatchRevocationsRequest>,
            ) -> Result<
                tonic::Response<tonic::codec::Streaming<super::RevocationEvent>>,
                tonic::Status,
            > {
                self.inner.ready().await.map_err(|e| {
                    tonic::Status::unknown(format!("service was not ready: {}", e.into()))
                })?;
                self.inner
                    .server_streaming(
                        request.into_request(),
                        http::uri::PathAndQuery::from_static(
                            "/evidenceos.v1.EvidenceOS/WatchRevocations",
                        ),
                        tonic::codec::ProstCodec::default(),
                    )
                    .await
            }
        }
    }
}
