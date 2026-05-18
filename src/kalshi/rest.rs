use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::debug;

use crate::config::AppConfig;
use crate::error::KalxError;

use super::auth::Signer;
use super::models::*;
use super::ws::KalshiWebSocketClient;

#[derive(Clone)]
pub struct KalshiClient {
    http: reqwest::Client,
    config: AppConfig,
    signer: Option<Arc<Signer>>,
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

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub async fn auth_check(&self) -> Result<AuthSummary> {
        if self.signer.is_none() {
            return Err(KalxError::MissingAuth.into());
        }

        let _: Value = self.authenticated_json(Method::GET, "/portfolio/balance", &[], None).await?;
        Ok(AuthSummary {
            ready: true,
            profile: self.config.profile.clone(),
            api_key_id_prefix: self
                .config
                .api_key_id
                .as_ref()
                .map(|value| value.chars().take(8).collect()),
            rest_base_url: self.config.active_profile().rest_base_url.clone(),
            private_key_path: self.config.private_key_path.clone(),
        })
    }

    pub async fn get_markets(&self, query: MarketsQuery) -> Result<MarketsResponse> {
        self.public_typed("/markets", &query_pairs([
            opt_pair("limit", query.limit),
            opt_pair("cursor", query.cursor),
            opt_pair("event_ticker", query.event_ticker),
            opt_pair("series_ticker", query.series_ticker),
            opt_pair("min_created_ts", query.min_created_ts),
            opt_pair("max_created_ts", query.max_created_ts),
            opt_pair("min_updated_ts", query.min_updated_ts),
            opt_pair("min_close_ts", query.min_close_ts),
            opt_pair("max_close_ts", query.max_close_ts),
            opt_pair("min_settled_ts", query.min_settled_ts),
            opt_pair("max_settled_ts", query.max_settled_ts),
            opt_pair("status", query.status),
            opt_pair("tickers", query.tickers),
            opt_pair("mve_filter", query.mve_filter),
        ])).await
    }

    pub async fn get_market(&self, ticker: &str) -> Result<MarketResponse> {
        self.public_typed(&format!("/markets/{ticker}"), &[]).await
    }

    pub async fn get_orderbook(&self, ticker: &str, depth: Option<u32>) -> Result<MarketOrderbookResponse> {
        self.public_typed(&format!("/markets/{ticker}/orderbook"), &query_pairs([opt_pair("depth", depth)])).await
    }

    pub async fn get_trades(&self, query: TradesQuery) -> Result<TradesResponse> {
        self.public_typed("/markets/trades", &query_pairs([
            opt_pair("ticker", query.ticker),
            opt_pair("limit", Some(query.limit)),
            opt_pair("cursor", query.cursor),
            opt_pair("min_ts", query.min_ts),
            opt_pair("max_ts", query.max_ts),
        ])).await
    }

    pub async fn get_market_candles(&self, series_ticker: &str, ticker: &str, query: CandlesQuery) -> Result<MarketCandlesResponse> {
        self.public_typed(
            &format!("/series/{series_ticker}/markets/{ticker}/candlesticks"),
            &query_pairs([
                opt_pair("start_ts", Some(query.start_ts)),
                opt_pair("end_ts", Some(query.end_ts)),
                opt_pair("period_interval", Some(query.period_interval)),
                opt_pair("include_latest_before_start", query.include_latest_before_start),
            ]),
        )
        .await
    }

    pub async fn get_events(&self, query: EventsQuery) -> Result<EventsResponse> {
        self.public_typed("/events", &query_pairs([
            opt_pair("limit", Some(query.limit)),
            opt_pair("cursor", query.cursor),
            opt_pair("status", query.status),
            opt_pair("series_ticker", query.series_ticker),
            opt_pair("with_nested_markets", Some(query.with_nested_markets)),
            opt_pair("with_milestones", Some(query.with_milestones)),
            opt_pair("min_updated_ts", query.min_updated_ts),
            opt_pair("min_close_ts", query.min_close_ts),
        ])).await
    }

    pub async fn get_event(&self, ticker: &str, nested_markets: bool) -> Result<EventResponse> {
        self.public_typed(
            &format!("/events/{ticker}"),
            &query_pairs([opt_pair("with_nested_markets", Some(nested_markets))]),
        )
        .await
    }

    pub async fn get_series_list(&self, query: SeriesQuery) -> Result<SeriesListResponse> {
        self.public_typed("/series", &query_pairs([
            opt_pair("category", query.category),
            opt_pair("tags", query.tags),
            opt_pair("include_volume", Some(query.include_volume)),
            opt_pair("include_product_metadata", Some(query.include_product_metadata)),
            opt_pair("min_updated_ts", query.min_updated_ts),
        ])).await
    }

    pub async fn get_series(&self, ticker: &str, include_volume: bool) -> Result<SeriesResponse> {
        self.public_typed(
            &format!("/series/{ticker}"),
            &query_pairs([opt_pair("include_volume", Some(include_volume))]),
        )
        .await
    }

    pub async fn get_exchange_status(&self) -> Result<Value> {
        self.public_json(Method::GET, "/exchange/status", &[], None).await
    }

    pub async fn get_exchange_schedule(&self) -> Result<ExchangeScheduleResponse> {
        self.public_typed("/exchange/schedule", &[]).await
    }

    pub async fn get_balance(&self, subaccount: Option<u32>) -> Result<BalanceResponse> {
        self.authenticated_typed("/portfolio/balance", &query_pairs([opt_pair("subaccount", subaccount)])).await
    }

    pub async fn get_positions(&self, query: PositionsQuery) -> Result<PositionsResponse> {
        self.authenticated_typed("/portfolio/positions", &query_pairs([
            opt_pair("ticker", query.ticker),
            opt_pair("event_ticker", query.event_ticker),
            opt_pair("count_filter", query.count_filter),
            opt_pair("limit", Some(query.limit)),
            opt_pair("cursor", query.cursor),
            opt_pair("subaccount", query.subaccount),
        ])).await
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
        self.authenticated_typed("/portfolio/fills", &query_pairs([
            opt_pair("ticker", ticker),
            opt_pair("order_id", order_id),
            opt_pair("limit", Some(limit)),
            opt_pair("cursor", cursor),
            opt_pair("min_ts", min_ts),
            opt_pair("max_ts", max_ts),
            opt_pair("subaccount", subaccount),
        ])).await
    }

    pub async fn get_settlements(&self, query: SettlementsQuery) -> Result<SettlementsResponse> {
        self.authenticated_typed("/portfolio/settlements", &query_pairs([
            opt_pair("ticker", query.ticker),
            opt_pair("event_ticker", query.event_ticker),
            opt_pair("limit", Some(query.limit)),
            opt_pair("cursor", query.cursor),
            opt_pair("min_ts", query.min_ts),
            opt_pair("max_ts", query.max_ts),
            opt_pair("subaccount", query.subaccount),
        ])).await
    }

    pub async fn get_orders(&self, query: OrdersQuery) -> Result<OrdersResponse> {
        self.authenticated_typed("/portfolio/orders", &query_pairs([
            opt_pair("ticker", query.ticker),
            opt_pair("event_ticker", query.event_ticker),
            opt_pair("status", query.status),
            opt_pair("limit", Some(query.limit)),
            opt_pair("cursor", query.cursor),
            opt_pair("min_ts", query.min_ts),
            opt_pair("max_ts", query.max_ts),
            opt_pair("subaccount", query.subaccount),
        ])).await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<OrderResponse> {
        self.authenticated_typed(&format!("/portfolio/orders/{order_id}"), &[]).await
    }

    pub async fn create_order(&self, payload: &CreateOrderRequest) -> Result<OrderResponse> {
        self.authenticated_typed_with_body(Method::POST, "/portfolio/orders", &[], Some(serde_json::to_value(payload)?)).await
    }

    pub async fn cancel_order(&self, order_id: &str, subaccount: Option<u32>) -> Result<CancelOrderResponse> {
        self.authenticated_typed_with_body(
            Method::DELETE,
            &format!("/portfolio/orders/{order_id}"),
            &query_pairs([opt_pair("subaccount", subaccount)]),
            None,
        )
        .await
    }

    pub async fn amend_order(&self, order_id: &str, payload: &AmendOrderRequest) -> Result<AmendOrderResponse> {
        self.authenticated_typed_with_body(
            Method::POST,
            &format!("/portfolio/orders/{order_id}/amend"),
            &[],
            Some(serde_json::to_value(payload)?),
        )
        .await
    }

    pub async fn public_json_path(&self, path: &str) -> Result<Value> {
        let (path_only, query) = split_path_and_query(path);
        self.public_json(Method::GET, &path_only, &query, None).await
    }

    pub async fn authenticated_json_path(&self, path: &str) -> Result<Value> {
        let (path_only, query) = split_path_and_query(path);
        self.authenticated_json(Method::GET, &path_only, &query, None).await
    }

    pub async fn connect_websocket(&self) -> Result<KalshiWebSocketClient> {
        let signer = self.signer.as_ref().ok_or(KalxError::MissingAuth)?;
        KalshiWebSocketClient::connect(self.config.active_profile().ws_base_url.as_str(), signer.as_ref()).await
    }

    async fn public_typed<T: DeserializeOwned>(&self, path: &str, query: &[(String, String)]) -> Result<T> {
        Ok(serde_json::from_value(self.public_json(Method::GET, path, query, None).await?)?)
    }

    async fn authenticated_typed<T: DeserializeOwned>(&self, path: &str, query: &[(String, String)]) -> Result<T> {
        Ok(serde_json::from_value(self.authenticated_json(Method::GET, path, query, None).await?)?)
    }

    async fn authenticated_typed_with_body<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<T> {
        Ok(serde_json::from_value(self.authenticated_json(method, path, query, body).await?)?)
    }

    pub async fn public_json(&self, method: Method, path: &str, query: &[(String, String)], body: Option<Value>) -> Result<Value> {
        self.send_json(method, path, query, body, false).await
    }

    pub async fn authenticated_json(&self, method: Method, path: &str, query: &[(String, String)], body: Option<Value>) -> Result<Value> {
        self.send_json(method, path, query, body, true).await
    }

    async fn send_json(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
        require_auth: bool,
    ) -> Result<Value> {
        let url = format!("{}{}", self.config.active_profile().rest_base_url, path);
        let signed_path = reqwest::Url::parse(&url)
            .context("failed to parse request url for signing")?
            .path()
            .to_string();
        let mut request = self.http.request(method.clone(), &url).query(query);

        if let Some(body) = body {
            request = request.json(&body);
        }

        if require_auth {
            let signer = self.signer.as_ref().ok_or(KalxError::MissingAuth)?;
            let timestamp = Utc::now().timestamp_millis().to_string();
            for (key, value) in signer.auth_headers(&timestamp, &method, &signed_path)? {
                request = request.header(key, value);
            }
        }

        debug!(method = %method, path, signed_path, query = ?query, auth = require_auth, "sending request");
        let response = request.send().await.context("request failed")?;
        let status = response.status();
        let body = response.text().await.context("failed to read response body")?;

        if !status.is_success() {
            return Err(http_error(status, &body));
        }

        serde_json::from_str(&body).with_context(|| format!("failed to parse response json from {path}"))
    }
}

fn http_error(status: StatusCode, body: &str) -> anyhow::Error {
    anyhow!("http {}: {}", status.as_u16(), body)
}

pub fn query_pairs<const N: usize>(pairs: [Option<(String, String)>; N]) -> Vec<(String, String)> {
    pairs.into_iter().flatten().collect()
}

pub fn opt_pair<T: ToString>(key: &str, value: Option<T>) -> Option<(String, String)> {
    value.map(|value| (key.to_string(), value.to_string()))
}

pub fn split_path_and_query(path: &str) -> (String, Vec<(String, String)>) {
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

#[cfg(test)]
mod tests {
    use super::split_path_and_query;

    #[test]
    fn splits_query_components() {
        let (path, query) = split_path_and_query("/markets?limit=5&status=open");
        assert_eq!(path, "/markets");
        assert_eq!(query, vec![
            ("limit".to_string(), "5".to_string()),
            ("status".to_string(), "open".to_string()),
        ]);
    }
}
