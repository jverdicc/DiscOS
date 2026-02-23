use discos_client::{pb, ClientAuth, ClientConnectConfig, ClientTlsOptions, DiscosClient};
use evidenceos_auth_protocol::{sign_hmac_sha256, signing_material, HMAC_SHA256_TEST_VECTORS};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::{Certificate, Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};

const CA_CERT_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MIIDEzCCAfugAwIBAgIUF3jKLuGh+1KQ9CtLqCp1msQ7MQAwDQYJKoZIhvcNAQEL
BQAwGTEXMBUGA1UEAwwORGlzY09TIFRlc3QgQ0EwHhcNMjYwMjIwMjIzODMzWhcN
MzYwMjE4MjIzODMzWjAZMRcwFQYDVQQDDA5EaXNjT1MgVGVzdCBDQTCCASIwDQYJ
KoZIhvcNAQEBBQADggEPADCCAQoCggEBAKUxtm1MxpXbKWhvnMCiccRbngVilH0/
3+L52vWaG8omC2FgHuvYFltwVWUowxnKwerI40DCsKdj2GSXTGCmsW01cct167Tu
GqOtVbGvJ+YGNrrdkekDM3hsjPuBUUu2WoTNe1OOr8FQXAh4HGwKbn8nYppKeBqe
XCqyd+qXNxxcgeeqEFVuaTyi2kjDuP0nL628P80Z4kGkUvosi6ndwju2P3N7HXwg
YOovBd/884ytifwt5TRcLbEqejZP3+P4pbv8S1rb4w1BjN3W21LjmGV6GUEsY/Ni
Z9KTI98bGfJXmedMVagHBv9k2JHVxJVdIAzAj0IkR/p/zTOiiXuowLsCAwEAAaNT
MFEwHQYDVR0OBBYEFNqMcNz3wUYFk8kusMVttwl+F1qUMB8GA1UdIwQYMBaAFNqM
cNz3wUYFk8kusMVttwl+F1qUMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQEL
BQADggEBACjEkny4VQIt65jifUcHuWfvrDSNpIjYsry9OwMS1m+4klAGCwAFkCio
WUA6fRgklgkZCambQU2OHhWK2eb0K0BhpG4qzq4gA965LOM8fdmnn2Sbh1qofwED
ajzDrKnKfV7nkKErCChPNOaxXdrul7TpT7N6NAzPLW4pkUZ9t/2eHTgdveeWK8jE
h0JrffppyPEi9QJOvRG3lAThWacU7fX4MkberSc2kr/8QHcQmGw5Y7cR3SappBt+
cqh27t/yaBHod4uDg6RjVMxcXjK3M3sAe/BHjMTpMnHez3cXfurwGi24P6a2dt7c
k2IC0Jh5TP3QGkcIMhVRVNqY67ETdpA=
-----END CERTIFICATE-----"#;
const SERVER_CERT_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MIIDKDCCAhCgAwIBAgIUN2nFLOTKE6r5tkuThGdHt8KE7ckwDQYJKoZIhvcNAQEL
BQAwGTEXMBUGA1UEAwwORGlzY09TIFRlc3QgQ0EwHhcNMjYwMjIwMjIzODM0WhcN
MzYwMjE4MjIzODM0WjAUMRIwEAYDVQQDDAlsb2NhbGhvc3QwggEiMA0GCSqGSIb3
DQEBAQUAA4IBDwAwggEKAoIBAQCxlNTiPkdm/GwXFTZMxeh3uyy7BgERac2kiXfN
MsfBL/4H4ZoqU7RPOe2kHKK1pejjRpudo7AOH15G52lPKvCkRzHdknK7hwNpQ2An
+Vg2NOoEbJPrHapvxyEe/AayHpS/TMxRcd0JsbGZfpEbXOR6H3efhEudviS5lhLG
/30XgUSeD9OCBg4dGS8m9nFhf3Pn5iqCqLW02uo3Pk4TpU0BxCu0JF+33x9je7YW
r1N1TdMMSSdJ9f9exeVafyGD2fii8I4Yxhy3x+/4keNvjqj6ZlRQh1loYyIAObfV
2euaXqWym0F6GqgqD5ZdpbDqfpYvohBwaMfdNTF9mPa+f2AbAgMBAAGjbTBrMBQG
A1UdEQQNMAuCCWxvY2FsaG9zdDATBgNVHSUEDDAKBggrBgEFBQcDATAdBgNVHQ4E
FgQU3QNd+JZ8PFdYWVkkab2C9LktMDowHwYDVR0jBBgwFoAU2oxw3PfBRgWTyS6w
xW23CX4XWpQwDQYJKoZIhvcNAQELBQADggEBAGWywfwHfY5iZsBXLCKbuIEgHmNp
RtJ9W/k6fe1zxHmI5zQtP5nwfithcMSH2FjIKtDzuj1Kp/MkuuYhnIhm7kyjibpW
r8KdlRiX/s8jbUBpBwpOCNMNyBSOY89q8VCrMR7wvlJx9PBmf2+vZTdY8T6zD57C
zagGBid4ZfIQZETvSEBfwV8Gk9chCBVyqZTRDIahlLP6W3YgmMHF0jpqjZU9Gypq
jfSzjcDyqz+BTVd88RjjR84ilowB9EniAZssezCcRrSV0Xt3dgCwX65v4npxXHWP
CCKiI/rvjM0vURK/Dhfa2GxNZy6aitVqwUGYDYA+m+z8C0NdcCYdfrz2DaI=
-----END CERTIFICATE-----"#;
const SERVER_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCxlNTiPkdm/GwX
FTZMxeh3uyy7BgERac2kiXfNMsfBL/4H4ZoqU7RPOe2kHKK1pejjRpudo7AOH15G
52lPKvCkRzHdknK7hwNpQ2An+Vg2NOoEbJPrHapvxyEe/AayHpS/TMxRcd0JsbGZ
fpEbXOR6H3efhEudviS5lhLG/30XgUSeD9OCBg4dGS8m9nFhf3Pn5iqCqLW02uo3
Pk4TpU0BxCu0JF+33x9je7YWr1N1TdMMSSdJ9f9exeVafyGD2fii8I4Yxhy3x+/4
keNvjqj6ZlRQh1loYyIAObfV2euaXqWym0F6GqgqD5ZdpbDqfpYvohBwaMfdNTF9
mPa+f2AbAgMBAAECggEAVM1Wph4ImgkkExfZoOsHuxmi4EMbQnYMsqzFDbRYyQxC
mv/kz56+ZfErZ2oXV3h90P8YdDzFZ9EaA/EhjKFcXo7zJLT5Xk04101J/3zm36gA
MxGox6gYip2E1xeSmP/al8o0bhZacHUvczYKVI9VOm6JgR87vv4c6pVcrTQ3x/9y
/qYEwHw9OZoTNdQL2hYgYDX02CZWe6h1kiyIoTUdIF44Pvf8+Y5A9hfe9LK2qoIe
9MFbhjDNBBwvx4Hj6XLXuA1rca9V4pXgmNufrqp/T5lI8qtlhRuxlEbrkJrsQSG6
s7Pu+30Esoj44RKvWWyLwZfKxBsLEylw72b6j36n0QKBgQDdLGSI8/haIzc0LBZb
Fw4pU27R5a8bjOMkv+c456pknIjU8weiIM2E3cDdGFf8jHuc/wuMGu2EAYCn0vpO
4SrYjJPDnEG+1hBviCDRSyDQPhg3t0ChoxBjxTAKl96IyXsFUWY+OlgABlskkKcm
B8S4BfUrFNs1kkBn7q9toK9ekQKBgQDNizkIOHW6cIEQL93DNQpKjoZhVgrJcqVk
/wv4DLTl43KjPxiYCDXU+Pwq0Nbou8jlBO50tJYs3xY3JcOwhMwID1npZJ0CBM9p
+K3TgtXJs0QARbmDE4hr8zC2UDcEogsdr+NDEIvhCbU+JhviWxKXy3Bzxft5Yi/Q
92PKqFUB6wKBgA8gNBfxp3ByrBnTUgwUvZEx9YhBTwJxVi9zOFr57PtIgUse+8yo
taV6jPAR9CJ/cQzBnIaOaOP4PlY75YZze7ynkIt2KkDk3ubhxmzJ2IqlVH6q966W
Ok64c5ql3EA6l0E72eQzlUUbKiyMAAJn0ZpMPgIeQQee4uy4tCKpNJUhAoGBAKTp
vTqRQjPyLDs2jHEgzz7+l2blSAZVC8q6r3m3iDSihsnfx/XDMJ3Nn1Ui9isI76iA
imnsskkSPJrGm/m2spUM7BDMfwSto1TdB2qaoLkSMc8eIje+pkgmeMDuHxaChPSu
uGKIlhJaXaadoOW+OG699V2OTbQYSVEaDGD/KiU9AoGBALQciax5DtFxpWzZIhYU
xW74kWtZVmhU6aIAx2EHDRN3D2r5vgptTuCIJgJpKuzVl8ZNEAlsiFhNNlBd46ru
hB5Vzk0ieUs2ZnsptEYZNEp4TrC3WW/Ovf2fVoLb11oLdQEu7wrevP4t0OsyGRHc
Qp7ZtfnZIdjjrqzK9M2atAUE
-----END PRIVATE KEY-----"#;
const CLIENT_CERT_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MIIDFjCCAf6gAwIBAgIUN2nFLOTKE6r5tkuThGdHt8KE7cowDQYJKoZIhvcNAQEL
BQAwGTEXMBUGA1UEAwwORGlzY09TIFRlc3QgQ0EwHhcNMjYwMjIwMjIzODM0WhcN
MzYwMjE4MjIzODM0WjAYMRYwFAYDVQQDDA1kaXNjb3MtY2xpZW50MIIBIjANBgkq
hkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAlYqdN3fTFOWAMwGUt8bQX4n9tKyBi1rX
zD1LqsrP50h+224WHqHRWFUrT54+ytMvI51xDLF4C65h/yOj/qhiBDE7TyprXGXv
/E/P1/P++gsFzTCZwJa/Ca57PzHLyB+IEeybYXGMh5hY6ZertjbbvalUT3ipTCRD
nVu5oJEkjJisFB2pjt9urXnV1NlCLIrI2vmAoA9yfk7ZZN3Hq7lkc91rLMCJ5o7O
RCq/ljlBhFkJTAu9Psu4W/+fqL7KKB9XTRDcAye/JO7BOTW+b9VrtVxe7cjSR1ol
JpbOK7IHTpUfAbQrbc9war5ABmq+jj9PQbYIFZyYe59hxLczKTopOwIDAQABo1cw
VTATBgNVHSUEDDAKBggrBgEFBQcDAjAdBgNVHQ4EFgQU91Cf/edg3QmMD9iHcCLB
E3xji4gwHwYDVR0jBBgwFoAU2oxw3PfBRgWTyS6wxW23CX4XWpQwDQYJKoZIhvcN
AQELBQADggEBAH5NZ1stmU9q0MsgMx2AVauaD705BEvYUTVBNzpi/pyATgjzAYzX
ZN8IbCSlbZ4hiX+JoM8MXiPAtEdqLG/AZcsN7QtWAY6m+ZKi/ACAuqs3jhrT4Iiw
VAsVKFZD2ryq4kBPzUOKr2ecUx6o4pbH87NaVq5o3DXpmMiWm9WpgNrOxktWAYo3
sGHDkt1fBQp2Rk7unz/NAmGTVM187TJGO2327uw12+A756AR/2WcbBEc/D8xAQuG
/W8/bPvicCoS2s3uh0qeiomuDXRYknke3gvJ3MTlvFSaziOs+7Z1Ngs4vEuWnu84
Us7NA8cnh583hIl5nxBq3vglmbQw5B3qskw=
-----END CERTIFICATE-----"#;
const CLIENT_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCVip03d9MU5YAz
AZS3xtBfif20rIGLWtfMPUuqys/nSH7bbhYeodFYVStPnj7K0y8jnXEMsXgLrmH/
I6P+qGIEMTtPKmtcZe/8T8/X8/76CwXNMJnAlr8Jrns/McvIH4gR7JthcYyHmFjp
l6u2Ntu9qVRPeKlMJEOdW7mgkSSMmKwUHamO326tedXU2UIsisja+YCgD3J+Ttlk
3ceruWRz3WsswInmjs5EKr+WOUGEWQlMC70+y7hb/5+ovsooH1dNENwDJ78k7sE5
Nb5v1Wu1XF7tyNJHWiUmls4rsgdOlR8BtCttz3BqvkAGar6OP09BtggVnJh7n2HE
tzMpOik7AgMBAAECggEAIh97qX0PrvyBiOIK9/aHdM2NlW0psE1d4a549jOKRlne
DJ8tr/a3yOLCW8wdwvV2k7do5P8YB/5WQTx2PvVYTGGZiYKB9hcSsG3b3QjBvXfH
bp9w7ifX1tM4L17jA7riN0jY2d7ptMU4QLFJzu3srXYWlX3Qj93UifM0w+VqxCP2
1QuyOQ0oO7ufYU/wWVOJ1eRWmvpfnCTb6X391zeaVpx/BDMyqjMrR+A6l96aK0dC
P191Fmq1TbifQ999owXPc7m1GYPRIHWXK2L8EMq64GlgGuZTezZ24CIz9+PAjgAR
+wgS1+3PAzVnv9N+AJhx3G9azOO8bpdVrifPziUQwQKBgQDSVsPa4wK/uIfh6TC1
wRsiuxbzMoSxX3woqmweMH5eIBZ3/LtcAWHlCPQV9gwKUDNkpGgyvRtxxwHxr2r6
LgWqNj+NFjT63n5CuTKvg7dxk2eZreDol+18WRhVh6g6YfrcTSTGXY1ssa9YgMCC
/Yo0HT5cvtF/ZqBpc23UI7M8+wKBgQC2ASGi+M25t7106zdmgYnE5XtexAi4VGec
0mljV8mYkVakJ/8av/QyjtrUl3ft2BE+9ac/dflyA1jjP0Chf+8yX7bjhQxmPO0k
Fq25EV7jTXVu/i4k1n+iQwDWDit3FCcnzTxUNdA+0OOCbJ/5F563NMcc1vWqmgC8
s8LuVKSQwQKBgCJ3a0KL3mByKXoATyYJTZwEUj9psMqr3dmAC5Vq1tovod7pf/4U
j+kK7YxHtDNgbvt34UcnK78aIBxtZTc3oWAB4aoJ0IanPMNMO5z9FWs7/0b0ch1K
//RXWSByyUM/2O6OiY8jt0/vUc9L0b/lMedWP2jNL+ETfQeqjX2sl+tPAoGAQo8x
XOc+XP+78mkezobq+i2uK17njXmYlrpAPktZd0kXdVsHKSIvzNl/X6Ww/zM4Q10U
99lOCwr8U8bK/QTLVhG40YXTngQD+WGt0HNwzxGBs8CS4XfsH0v/n0h5Tsf49c5R
lL5FVOORgB33duHTck6DzqEyIFzHjjrzO7OKp0ECgYAGs1JlJ0W4pRD5+ypElWd3
j6QtnhLnkv3l9Sf/dC4M/XQHLPQXTAP3hylYfHTIg46JVxekLXvpdgUfVA6xSN6t
F+W+M5nngIXbLqkDOSV+Eue/xMmHUrbrah90BsZ7BjZNhtZAOEMY4hvjGvroJgaH
ZgnizYZYRSL1JniQBDz2gw==
-----END PRIVATE KEY-----"#;

#[derive(Clone)]
struct TestDaemon;

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for TestDaemon {
    async fn health(
        &self,
        _: Request<pb::HealthRequest>,
    ) -> Result<Response<pb::HealthResponse>, Status> {
        Ok(Response::new(pb::HealthResponse {
            status: "ok".into(),
        }))
    }

    async fn create_claim_v2(
        &self,
        _: Request<pb::CreateClaimV2Request>,
    ) -> Result<Response<pb::CreateClaimV2Response>, Status> {
        Ok(Response::new(pb::CreateClaimV2Response {
            claim_id: vec![1; 32],
            topic_id: vec![2; 32],
        }))
    }

    async fn commit_artifacts(
        &self,
        _: Request<pb::CommitArtifactsRequest>,
    ) -> Result<Response<pb::CommitArtifactsResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn commit_wasm(
        &self,
        _: Request<pb::CommitWasmRequest>,
    ) -> Result<Response<pb::CommitWasmResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn freeze(
        &self,
        _: Request<pb::FreezeRequest>,
    ) -> Result<Response<pb::FreezeResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn execute_claim_v2(
        &self,
        _: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn fetch_capsule(
        &self,
        _: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn get_signed_tree_head(
        &self,
        _: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn get_inclusion_proof(
        &self,
        _: Request<pb::GetInclusionProofRequest>,
    ) -> Result<Response<pb::GetInclusionProofResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn get_consistency_proof(
        &self,
        _: Request<pb::GetConsistencyProofRequest>,
    ) -> Result<Response<pb::GetConsistencyProofResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn revoke_claim(
        &self,
        _: Request<pb::RevokeClaimRequest>,
    ) -> Result<Response<pb::RevokeClaimResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    type WatchRevocationsStream = tokio_stream::Empty<Result<pb::RevocationEvent, Status>>;
    async fn watch_revocations(
        &self,
        _: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn get_server_info(
        &self,
        _: Request<pb::GetServerInfoRequest>,
    ) -> Result<Response<pb::GetServerInfoResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
    async fn get_public_key(
        &self,
        _: Request<pb::GetPublicKeyRequest>,
    ) -> Result<Response<pb::GetPublicKeyResponse>, Status> {
        Err(Status::unimplemented("not used in test"))
    }
}

#[test]
fn shared_auth_protocol_vectors_match() {
    for vector in HMAC_SHA256_TEST_VECTORS {
        let material = signing_material(vector.request_id, vector.path, vector.timestamp);
        let signature = sign_hmac_sha256(vector.secret, &material);
        assert_eq!(hex::encode(signature), vector.expected_signature_hex);
    }
}

async fn spawn_tls_server<F>(interceptor: F) -> std::net::SocketAddr
where
    F: tonic::service::Interceptor + Send + Sync + Clone + 'static,
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    let tls_config = ServerTlsConfig::new()
        .identity(Identity::from_pem(SERVER_CERT_PEM, SERVER_KEY_PEM))
        .client_ca_root(Certificate::from_pem(CA_CERT_PEM));

    tokio::spawn(async move {
        Server::builder()
            .tls_config(tls_config)
            .expect("server tls config")
            .add_service(pb::evidence_os_server::EvidenceOsServer::with_interceptor(
                TestDaemon,
                interceptor,
            ))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("serve tls");
    });

    addr
}

#[tokio::test]
async fn connects_with_tls_mtls_and_bearer_auth() {
    let addr = spawn_tls_server(|request: Request<()>| {
        let auth = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        match auth {
            Some("Bearer discos-token") => Ok(request),
            _ => Err(Status::unauthenticated("missing or invalid bearer auth")),
        }
    })
    .await;

    let mut config = ClientConnectConfig::with_endpoint(format!("https://localhost:{addr}"));
    config.tls = Some(ClientTlsOptions {
        ca_cert_pem: CA_CERT_PEM.as_bytes().to_vec(),
        domain_name: Some("localhost".into()),
        client_cert_pem: Some(CLIENT_CERT_PEM.as_bytes().to_vec()),
        client_key_pem: Some(CLIENT_KEY_PEM.as_bytes().to_vec()),
    });
    config.auth = Some(ClientAuth::BearerToken("discos-token".into()));

    let mut client = DiscosClient::connect_with_config(config)
        .await
        .expect("connect with tls+bearer");

    assert_eq!(client.health().await.expect("health call").status, "ok");
    assert_eq!(
        client
            .create_claim_v2(pb::CreateClaimV2Request {
                claim_name: "secure-claim".into(),
                metadata: None,
                signals: None,
                holdout_ref: "h".into(),
                epoch_size: 1,
                oracle_num_symbols: 1,
                access_credit: 1,
                oracle_id: "default".into(),
            })
            .await
            .expect("create claim")
            .claim_id,
        vec![1; 32]
    );
}

#[tokio::test]
async fn connects_with_tls_and_hmac_auth() {
    let shared_secret = b"discos-shared-secret".to_vec();
    let secret_for_server = shared_secret.clone();
    let addr = spawn_tls_server(move |request: Request<()>| {
        let request_id = request
            .metadata()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing request id"))?;

        let signature = request
            .metadata()
            .get("x-evidenceos-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing signature"))?;
        let signature_hex = signature
            .strip_prefix("sha256=")
            .ok_or_else(|| Status::unauthenticated("invalid signature prefix"))?;

        let timestamp = request
            .metadata()
            .get("x-evidenceos-timestamp")
            .and_then(|v| v.to_str().ok());

        let key_id = request
            .metadata()
            .get("x-evidenceos-key-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("missing key id"))?;
        if key_id != "k-prod" {
            return Err(Status::unauthenticated("invalid key id"));
        }

        let material = signing_material(request_id, request.uri().path(), timestamp);
        let expected = hex::encode(sign_hmac_sha256(&secret_for_server, &material));

        if expected != signature_hex {
            return Err(Status::unauthenticated("invalid signature"));
        }

        Ok(request)
    })
    .await;

    let mut config = ClientConnectConfig::with_endpoint(format!("https://localhost:{addr}"));
    config.tls = Some(ClientTlsOptions {
        ca_cert_pem: CA_CERT_PEM.as_bytes().to_vec(),
        domain_name: Some("localhost".into()),
        client_cert_pem: Some(CLIENT_CERT_PEM.as_bytes().to_vec()),
        client_key_pem: Some(CLIENT_KEY_PEM.as_bytes().to_vec()),
    });
    config.auth = Some(ClientAuth::HmacSha256 {
        key_id: "k-prod".into(),
        secret: shared_secret,
    });

    let mut client = DiscosClient::connect_with_config(config)
        .await
        .expect("connect with tls+hmac");

    client.health().await.expect("health call");
    client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "hmac-claim".into(),
            metadata: None,
            signals: None,
            holdout_ref: "h".into(),
            epoch_size: 1,
            oracle_num_symbols: 1,
            access_credit: 1,
            oracle_id: "default".into(),
        })
        .await
        .expect("create claim");
}
