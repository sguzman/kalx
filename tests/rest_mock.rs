use std::fs;

use kalx::config::AppConfig;
use kalx::kalshi::{KalshiClient, MarketsQuery};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::rand_core::OsRng;
use rsa::RsaPrivateKey;
use tempfile::tempdir;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn fetches_public_markets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/trade-api/v2/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [{ "ticker": "TEST-1", "status": "open", "title": "Test market" }],
            "cursor": ""
        })))
        .mount(&server)
        .await;

    let mut config = AppConfig::defaults();
    config.profiles.get_mut("demo").unwrap().rest_base_url = format!("{}/trade-api/v2", server.uri());
    let client = KalshiClient::new(config).unwrap();
    let response = client.get_markets(MarketsQuery {
        limit: Some(10),
        cursor: None,
        event_ticker: None,
        series_ticker: None,
        min_created_ts: None,
        max_created_ts: None,
        min_updated_ts: None,
        min_close_ts: None,
        max_close_ts: None,
        min_settled_ts: None,
        max_settled_ts: None,
        status: None,
        tickers: None,
        mve_filter: None,
    }).await.unwrap();
    assert_eq!(response.markets.len(), 1);
    assert_eq!(response.markets[0].ticker, "TEST-1");
}

#[tokio::test]
async fn sends_authenticated_balance_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/trade-api/v2/portfolio/balance"))
        .and(header_exists("KALSHI-ACCESS-KEY"))
        .and(header_exists("KALSHI-ACCESS-TIMESTAMP"))
        .and(header_exists("KALSHI-ACCESS-SIGNATURE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "balance": 1000,
            "portfolio_value": 1000,
            "updated_ts": 1716000000,
            "balance_breakdown": []
        })))
        .mount(&server)
        .await;

    let dir = tempdir().unwrap();
    let key_path = dir.path().join("test.pem");
    let mut rng = OsRng;
    let key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let pem = key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap();
    fs::write(&key_path, pem.as_bytes()).unwrap();

    let mut config = AppConfig::defaults();
    config.profiles.get_mut("demo").unwrap().rest_base_url = format!("{}/trade-api/v2", server.uri());
    config.api_key_id = Some("test-key".into());
    config.private_key_path = Some(key_path.display().to_string());

    let client = KalshiClient::new(config).unwrap();
    let balance = client.get_balance(None).await.unwrap();
    assert_eq!(balance.balance, 1000);
}
