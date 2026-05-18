use anyhow::{Context, Result};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use reqwest::Method;
use serde_json::Value;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};

use super::auth::Signer;
use super::models::{SubscriptionParams, SubscriptionRequest, WatchKind};

pub struct KalshiWebSocketClient {
    stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    next_id: u64,
}

impl KalshiWebSocketClient {
    pub async fn connect(ws_url: &str, signer: &Signer) -> Result<Self> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let path = "/trade-api/ws/v2";
        let mut request = ws_url.into_client_request().context("failed to build websocket request")?;
        for (key, value) in signer.auth_headers(&timestamp, &Method::GET, path)? {
            request.headers_mut().insert(key, value);
        }
        let (stream, _) = connect_async(request).await.context("failed to connect websocket")?;
        Ok(Self { stream, next_id: 1 })
    }

    pub async fn subscribe(&mut self, channels: Vec<String>, market_tickers: Option<Vec<String>>) -> Result<()> {
        let payload = SubscriptionRequest {
            id: self.next_id,
            cmd: "subscribe".to_string(),
            params: SubscriptionParams { channels, market_tickers },
        };
        self.next_id += 1;
        let text = serde_json::to_string(&payload)?;
        self.stream.send(Message::Text(text.into())).await?;
        Ok(())
    }

    pub async fn watch_loop(mut self, kind: WatchKind, market_ticker: Option<String>) -> Result<()> {
        let filter_ticker = market_ticker.clone();
        match kind {
            WatchKind::Market => {
                self.subscribe(
                    vec!["ticker".into(), "trade".into(), "market_lifecycle_v2".into()],
                    market_ticker.map(|ticker| vec![ticker]),
                ).await?;
            }
            WatchKind::Orderbook => {
                self.subscribe(
                    vec!["orderbook_delta".into()],
                    market_ticker.map(|ticker| vec![ticker]),
                ).await?;
            }
            WatchKind::Fills => {
                self.subscribe(vec!["fill".into()], None).await?;
            }
            WatchKind::Positions => {
                self.subscribe(vec!["market_positions".into()], None).await?;
            }
        }

        info!("websocket subscription started");
        while let Some(message) = self.stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    let value: Value = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text.to_string()));
                    if let Some(expected) = filter_ticker.as_deref() {
                        if !message_matches_market(&value, expected) {
                            continue;
                        }
                    }
                    println!("{}", serde_json::to_string(&value)?);
                }
                Ok(Message::Binary(_)) => {
                    debug!("ignoring binary websocket frame");
                }
                Ok(Message::Ping(payload)) => {
                    self.stream.send(Message::Pong(payload)).await?;
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Frame(_)) => {}
                Ok(Message::Close(frame)) => {
                    warn!(?frame, "websocket closed");
                    break;
                }
                Err(error) => return Err(error).context("websocket stream error"),
            }
        }
        Ok(())
    }
}

fn message_matches_market(value: &Value, expected: &str) -> bool {
    if value.get("type").and_then(Value::as_str) == Some("subscribed") {
        return true;
    }

    let direct = value
        .get("msg")
        .and_then(|msg| msg.get("market_ticker"))
        .and_then(Value::as_str);
    if direct == Some(expected) {
        return true;
    }

    let nested = value
        .get("msg")
        .and_then(|msg| msg.get("additional_metadata"))
        .and_then(|meta| meta.get("market_ticker"))
        .and_then(Value::as_str);
    nested == Some(expected)
}

#[cfg(test)]
mod tests {
    use super::{SubscriptionRequest, message_matches_market};
    use crate::kalshi::models::SubscriptionParams;
    use serde_json::json;

    #[test]
    fn serializes_subscription_request() {
        let payload = SubscriptionRequest {
            id: 1,
            cmd: "subscribe".to_string(),
            params: SubscriptionParams {
                channels: vec!["ticker".to_string()],
                market_tickers: Some(vec!["ABC".to_string()]),
            },
        };
        let text = serde_json::to_string(&payload).unwrap();
        assert!(text.contains("\"cmd\":\"subscribe\""));
        assert!(text.contains("\"ticker\"") || text.contains("\"market_tickers\""));
    }

    #[test]
    fn filters_market_messages() {
        let message = json!({
            "msg": {
                "market_ticker": "ABC"
            }
        });
        assert!(message_matches_market(&message, "ABC"));
        assert!(!message_matches_market(&message, "XYZ"));
    }
}
