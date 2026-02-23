use std::time::{Duration, Instant};

use discos_client::{ClientConnectConfig, DiscosClient};

#[tokio::test]
#[ignore = "network-dependent blackhole timing test"]
async fn connect_to_blackhole_endpoint_times_out() {
    let mut config = ClientConnectConfig::with_endpoint("http://10.255.255.1:65535");
    config.connect_timeout_ms = 200;
    config.request_timeout_ms = 200;

    let start = Instant::now();
    let result = DiscosClient::connect_with_config(config).await;
    let elapsed = start.elapsed();

    assert!(result.is_err(), "expected connection to fail");
    assert!(
        elapsed < Duration::from_secs(3),
        "connect should not hang indefinitely (elapsed {elapsed:?})"
    );
}
