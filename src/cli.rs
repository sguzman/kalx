use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Table,
    Json,
    Ndjson,
}

#[derive(Debug, Parser)]
#[command(name = "kalx", version, about = "Kalshi exchange CLI")]
pub struct Cli {
    #[arg(long)]
    pub config: Option<String>,
    #[arg(long)]
    pub env_file: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[arg(long)]
    pub log_json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Doctor,
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
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Check,
    Keys,
}

#[derive(Debug, Subcommand)]
pub enum ExchangeCommands {
    Status,
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
        limit_updated_since: Option<i64>,
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
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        nested_markets: bool,
        #[arg(long)]
        updated_since: Option<i64>,
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
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        series_ticker: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
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
    },
    Get {
        ticker: String,
    },
    Search {
        query: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 500)]
        limit: u32,
    },
    Orderbook {
        ticker: String,
        #[arg(long)]
        depth: Option<u32>,
    },
    Trades {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
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
    Fills {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        min_ts: Option<i64>,
        #[arg(long)]
        max_ts: Option<i64>,
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Positions {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        settlement_status: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        subaccount: Option<u32>,
    },
    Orders {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long)]
        event_ticker: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
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
