use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSummary {
    pub ready: bool,
    pub profile: String,
    pub api_key_id_prefix: Option<String>,
    pub rest_base_url: String,
    pub private_key_path: Option<String>,
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
    pub min_settled_ts: Option<i64>,
    pub max_settled_ts: Option<i64>,
    pub status: Option<String>,
    pub tickers: Option<String>,
    pub mve_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsQuery {
    pub limit: u32,
    pub cursor: Option<String>,
    pub status: Option<String>,
    pub series_ticker: Option<String>,
    pub with_nested_markets: bool,
    pub with_milestones: bool,
    pub min_updated_ts: Option<i64>,
    pub min_close_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesQuery {
    pub category: Option<String>,
    pub tags: Option<String>,
    pub include_volume: bool,
    pub include_product_metadata: bool,
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
    pub event_ticker: Option<String>,
    pub count_filter: Option<String>,
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
pub struct SettlementsQuery {
    pub ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
    pub min_ts: Option<i64>,
    pub max_ts: Option<i64>,
    pub subaccount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlesQuery {
    pub start_ts: i64,
    pub end_ts: i64,
    pub period_interval: u32,
    pub include_latest_before_start: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketsResponse {
    #[serde(default)]
    pub markets: Vec<Market>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResponse {
    pub market: Market,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventsResponse {
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub milestones: Vec<Value>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeriesListResponse {
    #[serde(default)]
    pub series: Vec<Series>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesResponse {
    pub series: Series,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradesResponse {
    #[serde(default)]
    pub trades: Vec<Trade>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FillsResponse {
    #[serde(default)]
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PositionsResponse {
    #[serde(default)]
    pub market_positions: Vec<Position>,
    #[serde(default)]
    pub event_positions: Vec<EventPosition>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrdersResponse {
    #[serde(default)]
    pub orders: Vec<Order>,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettlementsResponse {
    #[serde(default)]
    pub settlements: Vec<Settlement>,
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
pub struct ExchangeScheduleResponse {
    pub schedule: ExchangeSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExchangeSchedule {
    #[serde(default)]
    pub standard_hours: Vec<Value>,
    #[serde(default)]
    pub maintenance_windows: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrderbookResponse {
    pub orderbook_fp: Orderbook,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Orderbook {
    #[serde(default)]
    pub yes_dollars: Vec<(String, String)>,
    #[serde(default)]
    pub no_dollars: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketCandlesResponse {
    #[serde(default)]
    pub candlesticks: Vec<Candlestick>,
    pub adjusted_end_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Candlestick {
    pub end_period_ts: Option<i64>,
    pub yes_bid: Option<PriceBand>,
    pub yes_ask: Option<PriceBand>,
    pub price: Option<PriceStats>,
    pub volume_fp: Option<String>,
    pub open_interest_fp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceBand {
    pub open_dollars: Option<String>,
    pub low_dollars: Option<String>,
    pub high_dollars: Option<String>,
    pub close_dollars: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceStats {
    pub open_dollars: Option<String>,
    pub low_dollars: Option<String>,
    pub high_dollars: Option<String>,
    pub close_dollars: Option<String>,
    pub mean_dollars: Option<String>,
    pub previous_dollars: Option<String>,
    pub min_dollars: Option<String>,
    pub max_dollars: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Market {
    pub ticker: String,
    pub event_ticker: Option<String>,
    pub market_type: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub status: Option<String>,
    pub open_time: Option<String>,
    pub close_time: Option<String>,
    pub updated_time: Option<String>,
    pub created_time: Option<String>,
    pub yes_sub_title: Option<String>,
    pub no_sub_title: Option<String>,
    pub yes_bid_dollars: Option<String>,
    pub yes_ask_dollars: Option<String>,
    pub yes_bid_size_fp: Option<String>,
    pub yes_ask_size_fp: Option<String>,
    pub no_bid_dollars: Option<String>,
    pub no_ask_dollars: Option<String>,
    pub last_price_dollars: Option<String>,
    pub volume_fp: Option<String>,
    pub volume_24h_fp: Option<String>,
    pub open_interest_fp: Option<String>,
    pub liquidity_dollars: Option<String>,
    pub rules_primary: Option<String>,
    pub rules_secondary: Option<String>,
    pub result: Option<String>,
}

impl Market {
    pub fn matches(&self, query: &str) -> bool {
        [
            Some(self.ticker.as_str()),
            self.event_ticker.as_deref(),
            self.title.as_deref(),
            self.subtitle.as_deref(),
            self.yes_sub_title.as_deref(),
            self.no_sub_title.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| value.to_ascii_lowercase().contains(query))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Event {
    pub event_ticker: String,
    pub series_ticker: Option<String>,
    pub title: Option<String>,
    pub sub_title: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub last_updated_ts: Option<String>,
    #[serde(default)]
    pub markets: Vec<Market>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trade {
    pub trade_id: Option<String>,
    pub ticker: Option<String>,
    pub count_fp: Option<String>,
    pub yes_price_dollars: Option<String>,
    pub no_price_dollars: Option<String>,
    pub taker_side: Option<String>,
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fill {
    pub fill_id: Option<String>,
    pub trade_id: Option<String>,
    pub order_id: Option<String>,
    pub ticker: Option<String>,
    pub market_ticker: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    pub count_fp: Option<String>,
    pub yes_price_dollars: Option<String>,
    pub no_price_dollars: Option<String>,
    pub created_time: Option<String>,
    pub subaccount_number: Option<u32>,
    pub ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position {
    pub ticker: Option<String>,
    pub total_traded_dollars: Option<String>,
    #[serde(rename = "position_fp")]
    pub position: Option<String>,
    #[serde(rename = "market_exposure_dollars")]
    pub market_exposure: Option<String>,
    #[serde(rename = "fees_paid_dollars")]
    pub fees_paid: Option<String>,
    pub realized_pnl_dollars: Option<String>,
    pub last_updated_ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventPosition {
    pub event_ticker: Option<String>,
    pub total_cost_dollars: Option<String>,
    pub total_cost_shares_fp: Option<String>,
    pub event_exposure_dollars: Option<String>,
    pub realized_pnl_dollars: Option<String>,
    pub fees_paid_dollars: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settlement {
    pub ticker: Option<String>,
    pub event_ticker: Option<String>,
    pub position: Option<i64>,
    pub settlement_price: Option<i64>,
    pub settlement_price_dollars: Option<String>,
    pub realized_pnl: Option<i64>,
    pub realized_pnl_dollars: Option<String>,
    pub settled_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Order {
    pub order_id: Option<String>,
    pub user_id: Option<String>,
    pub client_order_id: Option<String>,
    pub ticker: Option<String>,
    pub side: Option<String>,
    pub action: Option<String>,
    pub outcome_side: Option<String>,
    pub book_side: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
    pub status: Option<String>,
    pub yes_price_dollars: Option<String>,
    pub no_price_dollars: Option<String>,
    pub fill_count_fp: Option<String>,
    pub remaining_count_fp: Option<String>,
    pub initial_count_fp: Option<String>,
    pub taker_fill_cost_dollars: Option<String>,
    pub maker_fill_cost_dollars: Option<String>,
    pub taker_fees_dollars: Option<String>,
    pub maker_fees_dollars: Option<String>,
    pub expiration_time: Option<String>,
    pub created_time: Option<String>,
    pub last_update_time: Option<String>,
    pub self_trade_prevention_type: Option<String>,
    pub order_group_id: Option<String>,
    pub cancel_order_on_pause: Option<bool>,
    pub subaccount_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderResponse {
    pub order: Order,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CancelOrderResponse {
    pub order: Order,
    pub reduced_by_fp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AmendOrderResponse {
    pub old_order: Order,
    pub order: Order,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrderRequest {
    pub ticker: String,
    pub side: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AmendOrderRequest {
    pub ticker: String,
    pub side: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub id: u64,
    pub cmd: String,
    pub params: SubscriptionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionParams {
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
pub enum WatchKind {
    Market,
    Orderbook,
    Fills,
    Positions,
}
