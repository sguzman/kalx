mod cli;
mod config;
mod error;
mod kalshi;
mod logging;
mod output;

use std::collections::HashSet;
use std::io::{self, Write};
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::{Generator, Shell, generate};
use cli::{
    ApiCommands, AuthCommands, Cli, Commands, ConfigCommands, EventCommands, ExchangeCommands,
    MarketCommands, OutputFormat, PortfolioCommands, SeriesCommands,
};
use config::AppConfig;
use kalshi::{
    AuthSummary, Event, EventsQuery, KalshiClient, Market, MarketsQuery, Order, OrdersQuery,
    Position, PositionsQuery, Series, SeriesQuery, TradesQuery,
};
use output::{
    print_json, print_ndjson_values, print_table_balance, print_table_events, print_table_fills,
    print_table_markets, print_table_orderbook, print_table_orders, print_table_positions,
    print_table_series, print_table_trades,
};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(cli.config.as_deref(), cli.env_file.as_deref(), cli.profile.as_deref())?;
    logging::init(&cli, &config)?;
    let client = KalshiClient::new(config.clone())?;

    match cli.command {
        Commands::Doctor => run_doctor(&config)?,
        Commands::Config { command } => run_config_command(&config, command)?,
        Commands::Auth { command } => run_auth_command(&client, cli.output, command).await?,
        Commands::Exchange { command } => run_exchange_command(&client, cli.output, command).await?,
        Commands::Series { command } => run_series_command(&client, cli.output, command).await?,
        Commands::Events { command } => run_events_command(&client, cli.output, command).await?,
        Commands::Markets { command } => run_markets_command(&client, cli.output, command).await?,
        Commands::Portfolio { command } => run_portfolio_command(&client, cli.output, command).await?,
        Commands::Api { command } => run_api_command(&client, cli.output, command).await?,
        Commands::Completions { shell } => generate_completions(shell),
    }

    Ok(())
}

fn run_doctor(config: &AppConfig) -> Result<()> {
    let auth = config.auth_state();
    println!("profile: {}", config.profile);
    println!("rest_base_url: {}", config.active_profile().rest_base_url);
    println!("ws_base_url: {}", config.active_profile().ws_base_url);
    println!("config_file: {}", config.loaded_config_path.as_deref().unwrap_or("<none>"));
    println!("env_file: {}", config.loaded_env_path.as_deref().unwrap_or("<none>"));
    println!("api_key_id: {}", auth.api_key_present);
    println!("private_key_path: {}", auth.private_key_present);
    println!("auth_ready: {}", auth.ready);
    Ok(())
}

fn run_config_command(config: &AppConfig, command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => print_json(config),
        ConfigCommands::Path => {
            if let Some(path) = &config.loaded_config_path {
                println!("{path}");
            } else {
                println!("<none>");
            }
            Ok(())
        }
    }
}

async fn run_auth_command(client: &KalshiClient, format: OutputFormat, command: AuthCommands) -> Result<()> {
    match command {
        AuthCommands::Check => {
            let summary = client.auth_check().await?;
            emit_auth_summary(format, &summary)
        }
        AuthCommands::Keys => {
            let value = client.authenticated_json("/api_keys", &[]).await?;
            emit_value(format, &value)
        }
    }
}

async fn run_exchange_command(client: &KalshiClient, format: OutputFormat, command: ExchangeCommands) -> Result<()> {
    match command {
        ExchangeCommands::Status => {
            let value = client.public_json("/exchange/status", &[]).await?;
            emit_value(format, &value)
        }
    }
}

async fn run_series_command(client: &KalshiClient, format: OutputFormat, command: SeriesCommands) -> Result<()> {
    match command {
        SeriesCommands::List { category, tags, include_volume, limit_updated_since } => {
            let query = SeriesQuery {
                category,
                tags,
                include_volume,
                min_updated_ts: limit_updated_since,
            };
            let response = client.get_series(query).await?;
            emit_serieses(format, &response.series)
        }
        SeriesCommands::Get { ticker, include_volume } => {
            let response = client.get_series_by_ticker(&ticker, include_volume).await?;
            emit_value(format, &serde_json::to_value(response.series)?)
        }
    }
}

async fn run_events_command(client: &KalshiClient, format: OutputFormat, command: EventCommands) -> Result<()> {
    match command {
        EventCommands::List { status, series_ticker, limit, cursor, nested_markets, updated_since } => {
            let query = EventsQuery {
                limit,
                cursor,
                status,
                series_ticker,
                with_nested_markets: nested_markets,
                min_updated_ts: updated_since,
            };
            let response = client.get_events(query).await?;
            emit_events(format, &response.events)
        }
        EventCommands::Get { ticker, nested_markets } => {
            let response = client.get_event(&ticker, nested_markets).await?;
            emit_value(format, &serde_json::to_value(response.event)?)
        }
        EventCommands::Markets { ticker } => {
            let response = client.get_markets(MarketsQuery {
                limit: Some(100),
                cursor: None,
                event_ticker: Some(ticker),
                series_ticker: None,
                min_created_ts: None,
                max_created_ts: None,
                min_updated_ts: None,
                min_close_ts: None,
                max_close_ts: None,
                status: None,
                tickers: None,
            }).await?;
            emit_markets(format, &response.markets)
        }
    }
}

async fn run_markets_command(client: &KalshiClient, format: OutputFormat, command: MarketCommands) -> Result<()> {
    match command {
        MarketCommands::List { status, event_ticker, series_ticker, limit, cursor, min_created_ts, max_created_ts, min_updated_ts, min_close_ts, max_close_ts, tickers } => {
            let response = client.get_markets(MarketsQuery {
                limit: Some(limit),
                cursor,
                event_ticker,
                series_ticker,
                min_created_ts,
                max_created_ts,
                min_updated_ts,
                min_close_ts,
                max_close_ts,
                status,
                tickers,
            }).await?;
            emit_markets(format, &response.markets)
        }
        MarketCommands::Get { ticker } => {
            let response = client.get_market(&ticker).await?;
            emit_value(format, &serde_json::to_value(response.market)?)
        }
        MarketCommands::Search { query, status, limit } => {
            let response = client.get_markets(MarketsQuery {
                limit: Some(limit.max(1)),
                cursor: None,
                event_ticker: None,
                series_ticker: None,
                min_created_ts: None,
                max_created_ts: None,
                min_updated_ts: None,
                min_close_ts: None,
                max_close_ts: None,
                status,
                tickers: None,
            }).await?;
            let lowered = query.to_ascii_lowercase();
            let filtered: Vec<Market> = response
                .markets
                .into_iter()
                .filter(|market| market.matches(&lowered))
                .collect();
            emit_markets(format, &filtered)
        }
        MarketCommands::Orderbook { ticker, depth } => {
            let response = client.get_orderbook(&ticker, depth).await?;
            match format {
                OutputFormat::Table => print_table_orderbook(&ticker, &response),
                _ => emit_value(format, &serde_json::to_value(response)?),
            }
        }
        MarketCommands::Trades { ticker, limit, cursor, min_ts, max_ts } => {
            let response = client.get_trades(TradesQuery { ticker, limit, cursor, min_ts, max_ts }).await?;
            emit_trades(format, &response.trades)
        }
        MarketCommands::RecentOpen { minutes, limit } => {
            let cutoff = Utc::now().timestamp() - i64::from(minutes) * 60;
            let response = client.get_markets(MarketsQuery {
                limit: Some(limit),
                cursor: None,
                event_ticker: None,
                series_ticker: None,
                min_created_ts: Some(cutoff),
                max_created_ts: None,
                min_updated_ts: None,
                min_close_ts: None,
                max_close_ts: None,
                status: Some("open".to_string()),
                tickers: None,
            }).await?;
            emit_markets(format, &response.markets)
        }
        MarketCommands::WatchOpen { interval_seconds, limit, once } => {
            watch_recent_open(client, format, interval_seconds, limit, once).await
        }
    }
}

async fn run_portfolio_command(client: &KalshiClient, format: OutputFormat, command: PortfolioCommands) -> Result<()> {
    match command {
        PortfolioCommands::Balance { subaccount } => {
            let response = client.get_balance(subaccount).await?;
            match format {
                OutputFormat::Table => print_table_balance(&response),
                _ => emit_value(format, &serde_json::to_value(response)?),
            }
        }
        PortfolioCommands::Fills { ticker, order_id, limit, cursor, min_ts, max_ts, subaccount } => {
            let response = client.get_fills(ticker, order_id, limit, cursor, min_ts, max_ts, subaccount).await?;
            emit_fills(format, &response.fills)
        }
        PortfolioCommands::Positions { ticker, settlement_status, limit, cursor, subaccount } => {
            let response = client.get_positions(PositionsQuery {
                ticker,
                settlement_status,
                limit,
                cursor,
                subaccount,
            }).await?;
            emit_positions(format, &response.market_positions)
        }
        PortfolioCommands::Orders { ticker, event_ticker, status, limit, cursor, min_ts, max_ts, subaccount } => {
            let response = client.get_orders(OrdersQuery {
                ticker,
                event_ticker,
                status,
                limit,
                cursor,
                min_ts,
                max_ts,
                subaccount,
            }).await?;
            emit_orders(format, &response.orders)
        }
    }
}

async fn run_api_command(client: &KalshiClient, format: OutputFormat, command: ApiCommands) -> Result<()> {
    match command {
        ApiCommands::Get { path, auth } => {
            let value = if auth {
                client.authenticated_json_path(&path).await?
            } else {
                client.public_json_path(&path).await?
            };
            emit_value(format, &value)
        }
    }
}

async fn watch_recent_open(
    client: &KalshiClient,
    format: OutputFormat,
    interval_seconds: u64,
    limit: u32,
    once: bool,
) -> Result<()> {
    let mut seen = HashSet::new();
    loop {
        let response = client
            .get_markets(MarketsQuery {
                limit: Some(limit),
                cursor: None,
                event_ticker: None,
                series_ticker: None,
                min_created_ts: None,
                max_created_ts: None,
                min_updated_ts: None,
                min_close_ts: None,
                max_close_ts: None,
                status: Some("open".to_string()),
                tickers: None,
            })
            .await?;

        let newly_seen: Vec<Market> = response
            .markets
            .into_iter()
            .filter(|market| seen.insert(market.ticker.clone()))
            .collect();

        if !newly_seen.is_empty() {
            info!(count = newly_seen.len(), "new open markets observed");
            emit_markets(format, &newly_seen)?;
        } else {
            debug!("no new open markets observed");
        }

        if once {
            return Ok(());
        }

        io::stdout().flush().ok();
        tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    }
}

fn emit_auth_summary(format: OutputFormat, summary: &AuthSummary) -> Result<()> {
    emit_value(format, &serde_json::to_value(summary)?)
}

fn emit_markets(format: OutputFormat, markets: &[Market]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_markets(markets),
        OutputFormat::Json => print_json(markets),
        OutputFormat::Ndjson => print_ndjson_values(markets),
    }
}

fn emit_events(format: OutputFormat, events: &[Event]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_events(events),
        OutputFormat::Json => print_json(events),
        OutputFormat::Ndjson => print_ndjson_values(events),
    }
}

fn emit_serieses(format: OutputFormat, series: &[Series]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_series(series),
        OutputFormat::Json => print_json(series),
        OutputFormat::Ndjson => print_ndjson_values(series),
    }
}

fn emit_trades(format: OutputFormat, trades: &[kalshi::Trade]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_trades(trades),
        OutputFormat::Json => print_json(trades),
        OutputFormat::Ndjson => print_ndjson_values(trades),
    }
}

fn emit_fills(format: OutputFormat, fills: &[kalshi::Fill]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_fills(fills),
        OutputFormat::Json => print_json(fills),
        OutputFormat::Ndjson => print_ndjson_values(fills),
    }
}

fn emit_positions(format: OutputFormat, positions: &[Position]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_positions(positions),
        OutputFormat::Json => print_json(positions),
        OutputFormat::Ndjson => print_ndjson_values(positions),
    }
}

fn emit_orders(format: OutputFormat, orders: &[Order]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_orders(orders),
        OutputFormat::Json => print_json(orders),
        OutputFormat::Ndjson => print_ndjson_values(orders),
    }
}

fn emit_value(format: OutputFormat, value: &serde_json::Value) -> Result<()> {
    match format {
        OutputFormat::Table => print_json(value),
        OutputFormat::Json => print_json(value),
        OutputFormat::Ndjson => {
            if let Some(items) = value.as_array() {
                print_ndjson_values(items)
            } else {
                print_json(value)
            }
        }
    }
}

fn generate_completions(shell: Shell) {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    print_completions(shell, &mut command, &name);
}

fn print_completions<G: Generator>(generator: G, command: &mut clap::Command, name: &str) {
    generate(generator, command, name, &mut io::stdout());
}
