use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Table,
    Json,
    Ndjson,
    Csv,
}

#[derive(Debug, Clone, Args)]
pub struct PaginationArgs {
    #[arg(long, default_value_t = 100)]
    pub limit: u32,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Clone, Args)]
pub struct SafetyArgs {
    #[arg(long)]
    pub live: bool,
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Parser)]
#[command(name = "kalx", version, about = "Kalshi exchange CLI")]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<String>,
    #[arg(long, global = true)]
    pub env_file: Option<String>,
    #[arg(long, global = true)]
    pub profile: Option<String>,
    #[arg(long, value_enum, default_value = "table", global = true)]
    pub output: OutputFormat,
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
    #[arg(long, global = true)]
    pub log_json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Doctor {
        #[arg(long)]
        auth_check: bool,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    Exchange {
        #[command(subcommand)]
        command: ExchangeCommands,
    },
    Series {
        #[command(subcommand)]
        command: SeriesCommands,
    },
    Events {
        #[command(subcommand)]
        command: EventCommands,
    },
    Markets {
        #[command(subcommand)]
        command: MarketCommands,
    },
    Portfolio {
        #[command(subcommand)]
        command: PortfolioCommands,
    },
    Orders {
        #[command(subcommand)]
        command: OrderCommands,
    },
    Watch {
        #[command(subcommand)]
        command: WatchCommands,
    },
    Export {
        #[command(subcommand)]
        command: ExportCommands,
    },
    Api {
        #[command(subcommand)]
        command: ApiCommands,
    },
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    Show,
    Path,
    Init {
        #[arg(long)]
        write: Option<String>,
    },
    Validate,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Check,
    Keys,
}

#[derive(Debug, Subcommand)]
pub enum ExchangeCommands {
    Status,
    Schedule,
}

#[derive(Debug, Subcommand)]
pub enum SeriesCommands {
    List {
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        tags: Option<String>,
        #[arg(long)]
        include_volume: bool,
        #[arg(long)]
        include_product_metadata: bool,
        #[arg(long)]
        updated_since: Option<i64>,
    },
    Get {
        ticker: String,
        #[arg(long)]
        include_volume: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum EventCommands {
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        nested_markets: bool,
        #[arg(long)]
        with_milestones: bool,
        #[arg(long)]
        updated_since: Option<i64>,
        #[arg(long)]
        min_close_ts: Option<i64>,
    },
    Get {
        ticker: String,
        #[arg(long)]
        nested_markets: bool,
    },
    Markets {
        ticker: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum MarketCommands {
    Search {
        query: String,
        #[arg(long)]
        status: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_created_ts: Option<i64>,
        #[arg(long)]
        max_created_ts: Option<i64>,
        #[arg(long)]
        min_updated_ts: Option<i64>,
        #[arg(long)]
        min_close_ts: Option<i64>,
        #[arg(long)]
        max_close_ts: Option<i64>,
        #[arg(long)]
        tickers: Option<String>,
        #[arg(long)]
        mve_filter: Option<String>,
    },
    Get {
        ticker: String,
    },
    Candles {
        series_ticker: String,
        ticker: String,
        #[arg(long)]
        start_ts: i64,
        #[arg(long)]
        end_ts: i64,
        #[arg(long, default_value_t = 60)]
        period_interval: u32,
        #[arg(long)]
        include_latest_before_start: bool,
    },
    Orderbook {
        ticker: String,
        #[arg(long)]
        depth: Option<u32>,
    },
    Trades {
        #[arg(long)]
        ticker: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
    },
    Watch {
        ticker: String,
    },
    RecentOpen {
        #[arg(long, default_value_t = 60)]
        minutes: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    WatchOpen {
        #[arg(long, default_value_t = 15)]
        interval_seconds: u64,
        #[arg(long, default_value_t = 200)]
        limit: u32,
        #[arg(long)]
        once: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PortfolioCommands {
    Balance {
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Positions {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        count_filter: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Fills {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Settlements {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
        #[arg(long)]
        subaccount: Option<u32>,
    },
}

#[derive(Debug, Subcommand)]
pub enum OrderCommands {
    List {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Get {
        order_id: String,
    },
    Create(CreateOrderArgs),
    Cancel {
        order_id: String,
        #[arg(long)]
        subaccount: Option<u32>,
        #[command(flatten)]
        safety: SafetyArgs,
    },
    CancelMarket {
        market_ticker: String,
        #[arg(long)]
        subaccount: Option<u32>,
        #[command(flatten)]
        safety: SafetyArgs,
    },
    Amend(AmendOrderArgs),
}

#[derive(Debug, Clone, Args)]
pub struct CreateOrderArgs {
    #[arg(long)]
    pub ticker: String,
    #[arg(long)]
    pub side: String,
    #[arg(long)]
    pub action: String,
    #[arg(long)]
    pub client_order_id: Option<String>,
    #[arg(long)]
    pub count: Option<u32>,
    #[arg(long)]
    pub count_fp: Option<String>,
    #[arg(long)]
    pub yes_price: Option<u32>,
    #[arg(long)]
    pub no_price: Option<u32>,
    #[arg(long)]
    pub yes_price_dollars: Option<String>,
    #[arg(long)]
    pub no_price_dollars: Option<String>,
    #[arg(long)]
    pub expiration_ts: Option<i64>,
    #[arg(long)]
    pub time_in_force: Option<String>,
    #[arg(long)]
    pub buy_max_cost: Option<i64>,
    #[arg(long)]
    pub post_only: bool,
    #[arg(long)]
    pub reduce_only: bool,
    #[arg(long)]
    pub sell_position_floor: Option<i64>,
    #[arg(long)]
    pub self_trade_prevention_type: Option<String>,
    #[arg(long)]
    pub order_group_id: Option<String>,
    #[arg(long)]
    pub cancel_order_on_pause: bool,
    #[arg(long)]
    pub subaccount: Option<u32>,
    #[arg(long)]
    pub exchange_index: Option<u32>,
    #[command(flatten)]
    pub safety: SafetyArgs,
}

#[derive(Debug, Clone, Args)]
pub struct AmendOrderArgs {
    pub order_id: String,
    #[arg(long)]
    pub ticker: String,
    #[arg(long)]
    pub side: String,
    #[arg(long)]
    pub action: String,
    #[arg(long)]
    pub subaccount: Option<u32>,
    #[arg(long)]
    pub client_order_id: Option<String>,
    #[arg(long)]
    pub updated_client_order_id: Option<String>,
    #[arg(long)]
    pub yes_price: Option<u32>,
    #[arg(long)]
    pub no_price: Option<u32>,
    #[arg(long)]
    pub yes_price_dollars: Option<String>,
    #[arg(long)]
    pub no_price_dollars: Option<String>,
    #[arg(long)]
    pub count: Option<u32>,
    #[arg(long)]
    pub count_fp: Option<String>,
    #[command(flatten)]
    pub safety: SafetyArgs,
}

#[derive(Debug, Subcommand)]
pub enum WatchCommands {
    Market { market_ticker: String },
    Orderbook { market_ticker: String },
    Fills,
    Positions,
}

#[derive(Debug, Subcommand)]
pub enum ExportCommands {
    Markets {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
    },
    Trades {
        #[arg(long)]
        ticker: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
    },
    Positions {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        count_filter: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Fills {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[command(flatten)]
        pagination: PaginationArgs,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
        #[arg(long)]
        subaccount: Option<u32>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ApiCommands {
    Get {
        path: String,
        #[arg(long)]
        auth: bool,
    },
}
