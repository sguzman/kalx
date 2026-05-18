use std::fs;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use reqwest::{Method, StatusCode};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::BlindedSigningKey;
use rsa::rand_core::OsRng;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use tracing::{debug, instrument};

use crate::config::AppConfig;
use crate::error::KalxError;

#[derive(Clone)]
pub struct KalshiClient {
    http: reqwest::Client,
    config: AppConfig,
    signer: Option<Arc<Signer>>,
}

#[derive(Debug, Serialize)]
pub struct AuthSummary {
    pub ready: bool,
    pub profile: String,
    pub api_key_id_prefix: Option<String>,
    pub rest_base_url: String,
}

#[derive(Clone)]
struct Signer {
    api_key_id: String,
    private_key: Arc<RsaPrivateKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub event_ticker: Option<String>,
    pub series_ticker: Option<String>,
    pub min_created_ts: Option<i64>,
    pub max_created_ts: Option<i64>,
    pub min_updated_ts: Option<i64>,
    pub min_close_ts: Option<i64>,
    pub max_close_ts: Option<i64>,
    pub status: Option<String>,
    pub tickers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsQuery {
    pub limit: u32,
    pub cursor: Option<String>,
    pub status: Option<String>,
    pub series_ticker: Option<String>,
    pub with_nested_markets: bool,
    pub min_updated_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesQuery {
    pub category: Option<String>,
    pub tags: Option<String>,
    pub include_volume: bool,
    pub min_updated_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesQuery {
    pub ticker: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
    pub min_ts: Option<i64>,
    pub max_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsQuery {
    pub ticker: Option<String>,
    pub settlement_status: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersQuery {
    pub ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub status: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
    pub min_ts: Option<i64>,
    pub max_ts: Option<i64>,
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<Market>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResponse {
    pub market: Market,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<Event>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesesResponse {
    pub series: Vec<Series>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesResponse {
    pub series: Series,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub trades: Vec<Trade>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillsResponse {
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsResponse {
    #[serde(default)]
    pub market_positions: Vec<Position>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
    pub updated_ts: i64,
    #[serde(default)]
    pub balance_breakdown: Vec<BalanceBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceBreakdown {
    pub exchange_index: Option<i64>,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrderbookResponse {
    pub orderbook_fp: Orderbook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    #[serde(default)]
    pub yes_dollars: Vec<(String, String)>,
    #[serde(default)]
    pub no_dollars: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub ticker: String,
    pub event_ticker: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub status: Option<String>,
    pub open_time: Option<String>,
    pub close_time: Option<String>,
    pub updated_time: Option<String>,
    pub created_time: Option<String>,
    pub yes_bid_dollars: Option<String>,
    pub yes_ask_dollars: Option<String>,
    pub no_bid_dollars: Option<String>,
    pub no_ask_dollars: Option<String>,
    pub last_price_dollars: Option<String>,
    pub volume_fp: Option<String>,
    pub open_interest_fp: Option<String>,
}

impl Market {
    pub fn matches(&self, query: &str) -> bool {
        self.ticker.to_ascii_lowercase().contains(query)
            || self
                .event_ticker
                .as_deref()
                .is_some_and(|value| value.to_ascii_lowercase().contains(query))
            || self
                .title
                .as_deref()
                .is_some_and(|value| value.to_ascii_lowercase().contains(query))
            || self
                .subtitle
                .as_deref()
                .is_some_and(|value| value.to_ascii_lowercase().contains(query))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_ticker: String,
    pub series_ticker: Option<String>,
    pub title: Option<String>,
    pub sub_title: Option<String>,
    pub category: Option<String>,
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub markets: Vec<Market>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub ticker: String,
    pub frequency: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub volume_fp: Option<String>,
    pub last_updated_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: Option<String>,
    pub ticker: Option<String>,
    pub count_fp: Option<String>,
    pub yes_price_dollars: Option<String>,
    pub no_price_dollars: Option<String>,
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub fill_id: Option<String>,
    pub market_ticker: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    pub count_fp: Option<String>,
    pub yes_price_dollars: Option<String>,
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub ticker: Option<String>,
    pub position: Option<String>,
    pub market_exposure: Option<String>,
    pub fees_paid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Option<String>,
    pub ticker: Option<String>,
    pub status: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    pub yes_price_dollars: Option<String>,
    pub remaining_count_fp: Option<String>,
}

impl KalshiClient {
    pub fn new(config: AppConfig) -> Result<Self> {
        let signer = match (&config.api_key_id, &config.private_key_path) {
            (Some(api_key_id), Some(private_key_path)) => Some(Arc::new(Signer::new(api_key_id, private_key_path)?)),
            _ => None,
        };

        Ok(Self {
            http: reqwest::Client::builder()
                .user_agent(format!("kalx/{}", env!("CARGO_PKG_VERSION")))
                .build()
                .context("failed to build http client")?,
            config,
            signer,
        })
    }

    pub async fn auth_check(&self) -> Result<AuthSummary> {
        if self.signer.is_none() {
            return Err(KalxError::MissingAuth.into());
        }

        let _: Value = self.authenticated_json("/portfolio/balance", &[]).await?;
        Ok(AuthSummary {
            ready: true,
            profile: self.config.profile.clone(),
            api_key_id_prefix: self
                .config
                .api_key_id
                .as_ref()
                .map(|value| value.chars().take(8).collect()),
            rest_base_url: self.config.active_profile().rest_base_url.clone(),
        })
    }

    pub async fn get_markets(&self, query: MarketsQuery) -> Result<MarketsResponse> {
        self.public_typed(
            "/markets",
            &query_pairs([
                opt_pair("limit", query.limit),
                opt_pair("cursor", query.cursor),
                opt_pair("event_ticker", query.event_ticker),
                opt_pair("series_ticker", query.series_ticker),
                opt_pair("min_created_ts", query.min_created_ts),
                opt_pair("max_created_ts", query.max_created_ts),
                opt_pair("min_updated_ts", query.min_updated_ts),
                opt_pair("min_close_ts", query.min_close_ts),
                opt_pair("max_close_ts", query.max_close_ts),
                opt_pair("status", query.status),
                opt_pair("tickers", query.tickers),
            ]),
        )
        .await
    }

    pub async fn get_market(&self, ticker: &str) -> Result<MarketResponse> {
        self.public_typed(&format!("/markets/{ticker}"), &[]).await
    }

    pub async fn get_orderbook(&self, ticker: &str, depth: Option<u32>) -> Result<MarketOrderbookResponse> {
        self.public_typed(&format!("/markets/{ticker}/orderbook"), &query_pairs([opt_pair("depth", depth)])).await
    }

    pub async fn get_trades(&self, query: TradesQuery) -> Result<TradesResponse> {
        self.public_typed(
            "/markets/trades",
            &query_pairs([
                opt_pair("ticker", query.ticker),
                opt_pair("limit", Some(query.limit)),
                opt_pair("cursor", query.cursor),
                opt_pair("min_ts", query.min_ts),
                opt_pair("max_ts", query.max_ts),
            ]),
        )
        .await
    }

    pub async fn get_events(&self, query: EventsQuery) -> Result<EventsResponse> {
        self.public_typed(
            "/events",
            &query_pairs([
                opt_pair("limit", Some(query.limit)),
                opt_pair("cursor", query.cursor),
                opt_pair("status", query.status),
                opt_pair("series_ticker", query.series_ticker),
                opt_pair("with_nested_markets", Some(query.with_nested_markets)),
                opt_pair("min_updated_ts", query.min_updated_ts),
            ]),
        )
        .await
    }

    pub async fn get_event(&self, ticker: &str, nested_markets: bool) -> Result<EventResponse> {
        self.public_typed(
            &format!("/events/{ticker}"),
            &query_pairs([opt_pair("with_nested_markets", Some(nested_markets))]),
        )
        .await
    }

    pub async fn get_series(&self, query: SeriesQuery) -> Result<SeriesesResponse> {
        self.public_typed(
            "/series",
            &query_pairs([
                opt_pair("category", query.category),
                opt_pair("tags", query.tags),
                opt_pair("include_volume", Some(query.include_volume)),
                opt_pair("min_updated_ts", query.min_updated_ts),
            ]),
        )
        .await
    }

    pub async fn get_series_by_ticker(&self, ticker: &str, include_volume: bool) -> Result<SeriesResponse> {
        self.public_typed(
            &format!("/series/{ticker}"),
            &query_pairs([opt_pair("include_volume", Some(include_volume))]),
        )
        .await
    }

    pub async fn get_balance(&self, subaccount: Option<u32>) -> Result<BalanceResponse> {
        self.authenticated_typed("/portfolio/balance", &query_pairs([opt_pair("subaccount", subaccount)])).await
    }

    pub async fn get_fills(
        &self,
        ticker: Option<String>,
        order_id: Option<String>,
        limit: u32,
        cursor: Option<String>,
        min_ts: Option<i64>,
        max_ts: Option<i64>,
        subaccount: Option<u32>,
    ) -> Result<FillsResponse> {
        self.authenticated_typed(
            "/portfolio/fills",
            &query_pairs([
                opt_pair("ticker", ticker),
                opt_pair("order_id", order_id),
                opt_pair("limit", Some(limit)),
                opt_pair("cursor", cursor),
                opt_pair("min_ts", min_ts),
                opt_pair("max_ts", max_ts),
                opt_pair("subaccount", subaccount),
            ]),
        )
        .await
    }

    pub async fn get_positions(&self, query: PositionsQuery) -> Result<PositionsResponse> {
        self.authenticated_typed(
            "/portfolio/positions",
            &query_pairs([
                opt_pair("ticker", query.ticker),
                opt_pair("settlement_status", query.settlement_status),
                opt_pair("limit", Some(query.limit)),
                opt_pair("cursor", query.cursor),
                opt_pair("subaccount", query.subaccount),
            ]),
        )
        .await
    }

    pub async fn get_orders(&self, query: OrdersQuery) -> Result<OrdersResponse> {
        self.authenticated_typed(
            "/portfolio/orders",
            &query_pairs([
                opt_pair("ticker", query.ticker),
                opt_pair("event_ticker", query.event_ticker),
                opt_pair("status", query.status),
                opt_pair("limit", Some(query.limit)),
                opt_pair("cursor", query.cursor),
                opt_pair("min_ts", query.min_ts),
                opt_pair("max_ts", query.max_ts),
                opt_pair("subaccount", query.subaccount),
            ]),
        )
        .await
    }

    pub async fn public_json_path(&self, path: &str) -> Result<Value> {
        let (path_only, query) = split_path_and_query(path);
        self.public_json(&path_only, &query).await
    }

    pub async fn authenticated_json_path(&self, path: &str) -> Result<Value> {
        let (path_only, query) = split_path_and_query(path);
        self.authenticated_json(&path_only, &query).await
    }

    pub async fn public_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        self.send_json(Method::GET, path, query, false).await
    }

    pub async fn authenticated_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        self.send_json(Method::GET, path, query, true).await
    }

    async fn public_typed<T: for<'de> Deserialize<'de>>(&self, path: &str, query: &[(String, String)]) -> Result<T> {
        Ok(serde_json::from_value(self.public_json(path, query).await?)?)
    }

    async fn authenticated_typed<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T> {
        Ok(serde_json::from_value(self.authenticated_json(path, query).await?)?)
    }

    #[instrument(skip(self), fields(method = %method, path))]
    async fn send_json(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        require_auth: bool,
    ) -> Result<Value> {
        let url = format!("{}{}", self.config.active_profile().rest_base_url, path);
        let mut request = self.http.request(method.clone(), &url).query(query);
        if require_auth {
            let signer = self.signer.as_ref().ok_or(KalxError::MissingAuth)?;
            let timestamp = Utc::now().timestamp_millis().to_string();
            let signature = signer.sign(&timestamp, method.as_str(), path)?;
            request = request
                .header("KALSHI-ACCESS-KEY", &signer.api_key_id)
                .header("KALSHI-ACCESS-TIMESTAMP", timestamp)
                .header("KALSHI-ACCESS-SIGNATURE", signature);
        }

        debug!(query = ?query, auth = require_auth, "sending request");
        let response = request.send().await.context("request failed")?;
        let status = response.status();
        let body = response.text().await.context("failed to read response body")?;

        if !status.is_success() {
            return Err(http_error(status, &body));
        }

        serde_json::from_str(&body).with_context(|| format!("failed to parse response json from {path}"))
    }
}

impl Signer {
    fn new(api_key_id: &str, private_key_path: &str) -> Result<Self> {
        let pem = fs::read_to_string(private_key_path)
            .with_context(|| format!("failed to read private key {private_key_path}"))?;
        let private_key = RsaPrivateKey::from_pkcs8_pem(&pem)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(&pem))
            .context("failed to decode private key as PKCS#8 or PKCS#1 PEM")?;

        Ok(Self {
            api_key_id: api_key_id.to_string(),
            private_key: Arc::new(private_key),
        })
    }

    fn sign(&self, timestamp: &str, method: &str, path: &str) -> Result<String> {
        let signing_key = BlindedSigningKey::<Sha256>::new((*self.private_key).clone());
        let payload = format!("{timestamp}{}{path}", method.to_ascii_uppercase());
        let mut rng = OsRng;
        let signature = signing_key.sign_with_rng(&mut rng, payload.as_bytes());
        Ok(STANDARD.encode(signature.to_vec()))
    }
}

fn http_error(status: StatusCode, body: &str) -> anyhow::Error {
    anyhow!("http {}: {}", status.as_u16(), body)
}

fn query_pairs<const N: usize>(pairs: [Option<(String, String)>; N]) -> Vec<(String, String)> {
    pairs.into_iter().flatten().collect()
}

fn opt_pair<T: ToString>(key: &str, value: Option<T>) -> Option<(String, String)> {
    value.map(|value| (key.to_string(), value.to_string()))
}

fn split_path_and_query(path: &str) -> (String, Vec<(String, String)>) {
    let mut parts = path.splitn(2, '?');
    let path_only = parts.next().unwrap_or(path).to_string();
    let query = parts
        .next()
        .map(|rest| {
            rest.split('&')
                .filter(|pair| !pair.is_empty())
                .map(|pair| {
                    let mut inner = pair.splitn(2, '=');
                    let key = inner.next().unwrap_or_default().to_string();
                    let value = inner.next().unwrap_or_default().to_string();
                    (key, value)
                })
                .collect()
        })
        .unwrap_or_default();
    (path_only, query)
}
