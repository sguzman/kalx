use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::{Generator, Shell, generate};
use kalx::cli::{
    AccountCommands, AmendOrderArgs, ApiCommands, AuthCommands, Cli, Commands, ConfigCommands,
    CreateOrderArgs, EventCommands, ExchangeCommands, ExportCommands, MarketCommands,
    OrderCommands, OutputFormat, PaginationArgs, PortfolioCommands, SeriesCommands, WatchCommands,
};
use kalx::config::AppConfig;
use kalx::kalshi::{
    AmendOrderRequest, CandlesQuery, CreateOrderRequest, Event, EventsQuery,
    Fill, KalshiClient, Market, MarketsQuery, Order, OrdersQuery, Position, PositionsQuery,
    Series, SeriesQuery, Settlement, SettlementsQuery, Trade, TradesQuery, WatchKind,
};
use kalx::logging;
use kalx::output::{
    print_csv_records, print_json, print_ndjson_values, print_table_balance, print_table_candles,
    print_table_events, print_table_fills, print_table_markets, print_table_orderbook,
    print_table_orders, print_table_positions, print_table_series, print_table_settlements,
    print_table_trades, print_value_csv,
};
use reqwest::Method;
use serde_json::{Value, json};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(cli.config.as_deref(), cli.env_file.as_deref(), cli.profile.as_deref())?;
    logging::init(&cli, &config)?;
    let client = KalshiClient::new(config.clone())?;

    match cli.command {
        Commands::Doctor { auth_check } => run_doctor(&client, auth_check).await?,
        Commands::Config { command } => run_config_command(&config, command)?,
        Commands::Auth { command } => run_auth_command(&client, cli.output, command).await?,
        Commands::Account { command } => run_account_command(&client, cli.output, command).await?,
        Commands::Exchange { command } => run_exchange_command(&client, cli.output, command).await?,
        Commands::Series { command } => run_series_command(&client, cli.output, command).await?,
        Commands::Events { command } => run_events_command(&client, cli.output, command).await?,
        Commands::Markets { command } => run_markets_command(&client, cli.output, command).await?,
        Commands::Portfolio { command } => run_portfolio_command(&client, cli.output, command).await?,
        Commands::Orders { command } => run_orders_command(&client, cli.output, command).await?,
        Commands::Watch { command } => run_watch_command(&client, command).await?,
        Commands::Export { command } => run_export_command(&client, cli.output, command).await?,
        Commands::Api { command } => run_api_command(&client, cli.output, command).await?,
        Commands::Completions { shell } => generate_completions(shell),
    }

    Ok(())
}

async fn run_doctor(client: &KalshiClient, auth_check: bool) -> Result<()> {
    let config = client.config();
    let auth = config.auth_state();
    let validation = config.validate();
    let mut report = json!({
        "profile": config.profile,
        "rest_base_url": config.active_profile().rest_base_url,
        "ws_base_url": config.active_profile().ws_base_url,
        "config_file": config.loaded_config_path.as_deref().unwrap_or("<none>"),
        "env_file": config.loaded_env_path.as_deref().unwrap_or("<none>"),
        "api_key_present": auth.api_key_present,
        "private_key_present": auth.private_key_present,
        "auth_ready": auth.ready,
        "validation": validation,
    });

    if auth_check && auth.ready {
        let auth_result = client.auth_check().await;
        report["auth_check"] = match auth_result {
            Ok(summary) => serde_json::to_value(summary)?,
            Err(error) => json!({ "ok": false, "error": error.to_string() }),
        };
    }

    print_json(&report)
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
        ConfigCommands::Init { write } => {
            if let Some(path) = write {
                fs::write(&path, AppConfig::default_config_toml())?;
                println!("{path}");
                Ok(())
            } else {
                print!("{}", AppConfig::default_config_toml());
                Ok(())
            }
        }
        ConfigCommands::Validate => print_json(&config.validate()),
    }
}

async fn run_auth_command(client: &KalshiClient, format: OutputFormat, command: AuthCommands) -> Result<()> {
    match command {
        AuthCommands::Check => emit_value(format, &serde_json::to_value(client.auth_check().await?)?),
        AuthCommands::Keys => {
            let value = client.authenticated_json(Method::GET, "/api_keys", &[], None).await?;
            emit_value(format, &value)
        }
    }
}

async fn run_account_command(client: &KalshiClient, format: OutputFormat, command: AccountCommands) -> Result<()> {
    match command {
        AccountCommands::Limits => emit_value(format, &serde_json::to_value(client.get_account_limits().await?)?),
        AccountCommands::EndpointCosts => {
            emit_value(format, &serde_json::to_value(client.get_endpoint_costs().await?)?)
        }
    }
}

async fn run_exchange_command(client: &KalshiClient, format: OutputFormat, command: ExchangeCommands) -> Result<()> {
    match command {
        ExchangeCommands::Status => emit_value(format, &client.get_exchange_status().await?),
        ExchangeCommands::Schedule => emit_value(format, &serde_json::to_value(client.get_exchange_schedule().await?)?),
    }
}

async fn run_series_command(client: &KalshiClient, format: OutputFormat, command: SeriesCommands) -> Result<()> {
    match command {
        SeriesCommands::List { category, tags, include_volume, include_product_metadata, updated_since } => {
            let response = client.get_series_list(SeriesQuery {
                category,
                tags,
                include_volume,
                include_product_metadata,
                min_updated_ts: updated_since,
            }).await?;
            emit_serieses(format, &response.series)
        }
        SeriesCommands::Get { ticker, include_volume } => {
            let response = client.get_series(&ticker, include_volume).await?;
            emit_value(format, &serde_json::to_value(response.series)?)
        }
    }
}

async fn run_events_command(client: &KalshiClient, format: OutputFormat, command: EventCommands) -> Result<()> {
    match command {
        EventCommands::List { status, series_ticker, pagination, nested_markets, with_milestones, updated_since, min_close_ts } => {
            let events = fetch_all_events(
                client,
                EventsQuery {
                    limit: pagination.limit,
                    cursor: pagination.cursor,
                    status,
                    series_ticker,
                    with_nested_markets: nested_markets,
                    with_milestones,
                    min_updated_ts: updated_since,
                    min_close_ts,
                },
                pagination.all,
            ).await?;
            emit_events(format, &events)
        }
        EventCommands::Get { ticker, nested_markets } => {
            let response = client.get_event(&ticker, nested_markets).await?;
            emit_value(format, &serde_json::to_value(response.event)?)
        }
        EventCommands::Markets { ticker } => {
            let markets = fetch_all_markets(
                client,
                MarketsQuery {
                    limit: Some(100),
                    cursor: None,
                    event_ticker: Some(ticker),
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
                },
                true,
            ).await?;
            emit_markets(format, &markets)
        }
    }
}

async fn run_markets_command(client: &KalshiClient, format: OutputFormat, command: MarketCommands) -> Result<()> {
    match command {
        MarketCommands::Search { query, status, pagination } => {
            let filtered = search_markets(client, &query, status, pagination).await?;
            emit_markets(format, &filtered)
        }
        MarketCommands::List { status, event_ticker, series_ticker, pagination, min_created_ts, max_created_ts, min_updated_ts, min_close_ts, max_close_ts, tickers, mve_filter } => {
            let markets = fetch_all_markets(
                client,
                MarketsQuery {
                    limit: Some(pagination.limit),
                    cursor: pagination.cursor,
                    event_ticker,
                    series_ticker,
                    min_created_ts,
                    max_created_ts,
                    min_updated_ts,
                    min_close_ts,
                    max_close_ts,
                    min_settled_ts: None,
                    max_settled_ts: None,
                    status,
                    tickers,
                    mve_filter,
                },
                pagination.all,
            ).await?;
            emit_markets(format, &markets)
        }
        MarketCommands::Get { ticker } => {
            let response = client.get_market(&ticker).await?;
            emit_value(format, &serde_json::to_value(response.market)?)
        }
        MarketCommands::Candles { series_ticker, ticker, start_ts, end_ts, period_interval, include_latest_before_start } => {
            let response = client.get_market_candles(
                &series_ticker,
                &ticker,
                CandlesQuery {
                    start_ts,
                    end_ts,
                    period_interval,
                    include_latest_before_start: Some(include_latest_before_start),
                },
            ).await?;
            match format {
                OutputFormat::Table => print_table_candles(&response.candlesticks),
                OutputFormat::Json => print_json(&response.candlesticks),
                OutputFormat::Ndjson => print_ndjson_values(&response.candlesticks),
                OutputFormat::Csv => print_csv_records(&response.candlesticks),
            }
        }
        MarketCommands::Orderbook { ticker, depth } => {
            let response = client.get_orderbook(&ticker, depth).await?;
            match format {
                OutputFormat::Table => print_table_orderbook(&ticker, &response),
                _ => emit_value(format, &serde_json::to_value(response)?),
            }
        }
        MarketCommands::Trades { ticker, pagination, min_ts, max_ts } => {
            let trades = fetch_all_trades(client, TradesQuery {
                ticker,
                limit: pagination.limit,
                cursor: pagination.cursor,
                min_ts,
                max_ts,
            }, pagination.all).await?;
            emit_trades(format, &trades)
        }
        MarketCommands::Watch { ticker } => run_watch_market(client, WatchKind::Market, Some(ticker)).await,
        MarketCommands::RecentOpen { minutes, limit, query } => {
            let cutoff = Utc::now().timestamp() - i64::from(minutes) * 60;
            let markets = fetch_all_markets(
                client,
                MarketsQuery {
                    limit: Some(limit),
                    cursor: None,
                    event_ticker: None,
                    series_ticker: None,
                    min_created_ts: Some(cutoff),
                    max_created_ts: None,
                    min_updated_ts: None,
                    min_close_ts: None,
                    max_close_ts: None,
                    min_settled_ts: None,
                    max_settled_ts: None,
                    status: Some("open".to_string()),
                    tickers: None,
                    mve_filter: None,
                },
                true,
            ).await?;
            let filtered = filter_markets_by_query(markets, query.as_deref());
            emit_markets(format, &filtered)
        }
        MarketCommands::WatchOpen { interval_seconds, limit, once, query } => {
            watch_recent_open(client, format, interval_seconds, limit, once, query.as_deref()).await
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
        PortfolioCommands::Positions { ticker, event_ticker, count_filter, pagination, subaccount } => {
            let positions = fetch_all_positions(client, PositionsQuery {
                ticker,
                event_ticker,
                count_filter,
                limit: pagination.limit,
                cursor: pagination.cursor,
                subaccount,
            }, pagination.all).await?;
            emit_positions(format, &positions)
        }
        PortfolioCommands::Fills { ticker, order_id, pagination, min_ts, max_ts, subaccount } => {
            let fills = fetch_all_fills(client, ticker, order_id, pagination, min_ts, max_ts, subaccount).await?;
            emit_fills(format, &fills)
        }
        PortfolioCommands::Settlements { ticker, event_ticker, pagination, min_ts, max_ts, subaccount } => {
            let settlements = fetch_all_settlements(client, SettlementsQuery {
                ticker,
                event_ticker,
                limit: pagination.limit,
                cursor: pagination.cursor,
                min_ts,
                max_ts,
                subaccount,
            }, pagination.all).await?;
            emit_settlements(format, &settlements)
        }
    }
}

async fn run_orders_command(client: &KalshiClient, format: OutputFormat, command: OrderCommands) -> Result<()> {
    match command {
        OrderCommands::List { ticker, event_ticker, status, pagination, min_ts, max_ts, subaccount } => {
            let orders = fetch_all_orders(client, OrdersQuery {
                ticker,
                event_ticker,
                status,
                limit: pagination.limit,
                cursor: pagination.cursor,
                min_ts,
                max_ts,
                subaccount,
            }, pagination.all).await?;
            emit_orders(format, &orders)
        }
        OrderCommands::Get { order_id } => emit_value(format, &serde_json::to_value(client.get_order(&order_id).await?)?),
        OrderCommands::Create(args) => run_create_order(client, format, args).await,
        OrderCommands::Cancel { order_id, subaccount, safety } => run_cancel_order(client, format, &order_id, subaccount, safety.live, safety.yes).await,
        OrderCommands::CancelMarket { market_ticker, subaccount, safety } => run_cancel_market(client, format, &market_ticker, subaccount, safety.live, safety.yes).await,
        OrderCommands::Amend(args) => run_amend_order(client, format, args).await,
    }
}

async fn run_watch_command(client: &KalshiClient, command: WatchCommands) -> Result<()> {
    match command {
        WatchCommands::Market { market_ticker } => run_watch_market(client, WatchKind::Market, Some(market_ticker)).await,
        WatchCommands::Orderbook { market_ticker } => run_watch_market(client, WatchKind::Orderbook, Some(market_ticker)).await,
        WatchCommands::Fills => run_watch_market(client, WatchKind::Fills, None).await,
        WatchCommands::Positions => run_watch_market(client, WatchKind::Positions, None).await,
    }
}

async fn run_export_command(client: &KalshiClient, format: OutputFormat, command: ExportCommands) -> Result<()> {
    match command {
        ExportCommands::Markets { status, event_ticker, series_ticker, pagination } => {
            let markets = fetch_all_markets(client, MarketsQuery {
                limit: Some(pagination.limit),
                cursor: pagination.cursor,
                event_ticker,
                series_ticker,
                min_created_ts: None,
                max_created_ts: None,
                min_updated_ts: None,
                min_close_ts: None,
                max_close_ts: None,
                min_settled_ts: None,
                max_settled_ts: None,
                status,
                tickers: None,
                mve_filter: None,
            }, true).await?;
            emit_markets(format, &markets)
        }
        ExportCommands::Trades { ticker, pagination, min_ts, max_ts } => {
            let trades = fetch_all_trades(client, TradesQuery { ticker, limit: pagination.limit, cursor: pagination.cursor, min_ts, max_ts }, true).await?;
            emit_trades(format, &trades)
        }
        ExportCommands::Positions { ticker, event_ticker, count_filter, pagination, subaccount } => {
            let positions = fetch_all_positions(client, PositionsQuery { ticker, event_ticker, count_filter, limit: pagination.limit, cursor: pagination.cursor, subaccount }, true).await?;
            emit_positions(format, &positions)
        }
        ExportCommands::Fills { ticker, order_id, pagination, min_ts, max_ts, subaccount } => {
            let fills = fetch_all_fills(client, ticker, order_id, pagination, min_ts, max_ts, subaccount).await?;
            emit_fills(format, &fills)
        }
    }
}

async fn run_api_command(client: &KalshiClient, format: OutputFormat, command: ApiCommands) -> Result<()> {
    match command {
        ApiCommands::Get { path, auth } => {
            let value = if auth { client.authenticated_json_path(&path).await? } else { client.public_json_path(&path).await? };
            emit_value(format, &value)
        }
    }
}

async fn run_create_order(client: &KalshiClient, format: OutputFormat, args: CreateOrderArgs) -> Result<()> {
    client.config().ensure_mutation_allowed(args.safety.live, args.safety.yes)?;
    let payload = CreateOrderRequest {
        ticker: args.ticker,
        side: args.side,
        action: args.action,
        client_order_id: args.client_order_id,
        count: args.count,
        count_fp: args.count_fp,
        yes_price: args.yes_price,
        no_price: args.no_price,
        yes_price_dollars: args.yes_price_dollars,
        no_price_dollars: args.no_price_dollars,
        expiration_ts: args.expiration_ts,
        time_in_force: args.time_in_force,
        buy_max_cost: args.buy_max_cost,
        post_only: Some(args.post_only),
        reduce_only: Some(args.reduce_only),
        sell_position_floor: args.sell_position_floor,
        self_trade_prevention_type: args.self_trade_prevention_type,
        order_group_id: args.order_group_id,
        cancel_order_on_pause: Some(args.cancel_order_on_pause),
        subaccount: args.subaccount,
        exchange_index: args.exchange_index,
    };
    if !args.safety.live {
        let preview = json!({ "dry_run": true, "operation": "orders.create", "payload": payload });
        return emit_value(format, &preview);
    }
    let response = client.create_order(&payload).await?;
    emit_value(format, &serde_json::to_value(response)?)
}

async fn run_cancel_order(client: &KalshiClient, format: OutputFormat, order_id: &str, subaccount: Option<u32>, live: bool, yes: bool) -> Result<()> {
    client.config().ensure_mutation_allowed(live, yes)?;
    if !live {
        return emit_value(format, &json!({
            "dry_run": true,
            "operation": "orders.cancel",
            "order_id": order_id,
            "subaccount": subaccount,
        }));
    }
    let response = client.cancel_order(order_id, subaccount).await?;
    emit_value(format, &serde_json::to_value(response)?)
}

async fn run_cancel_market(client: &KalshiClient, format: OutputFormat, market_ticker: &str, subaccount: Option<u32>, live: bool, yes: bool) -> Result<()> {
    client.config().ensure_mutation_allowed(live, yes)?;
    let orders = fetch_all_orders(client, OrdersQuery {
        ticker: Some(market_ticker.to_string()),
        event_ticker: None,
        status: Some("resting".to_string()),
        limit: 200,
        cursor: None,
        min_ts: None,
        max_ts: None,
        subaccount,
    }, true).await?;

    let ids: Vec<String> = orders.iter().filter_map(|order| order.order_id.clone()).collect();
    if !live {
        return emit_value(format, &json!({
            "dry_run": true,
            "operation": "orders.cancel-market",
            "market_ticker": market_ticker,
            "order_ids": ids,
        }));
    }

    let mut cancelled = Vec::new();
    for order_id in ids {
        cancelled.push(client.cancel_order(&order_id, subaccount).await?);
    }
    emit_value(format, &serde_json::to_value(cancelled)?)
}

async fn run_amend_order(client: &KalshiClient, format: OutputFormat, args: AmendOrderArgs) -> Result<()> {
    client.config().ensure_mutation_allowed(args.safety.live, args.safety.yes)?;
    let payload = AmendOrderRequest {
        ticker: args.ticker,
        side: args.side,
        action: args.action,
        subaccount: args.subaccount,
        client_order_id: args.client_order_id,
        updated_client_order_id: args.updated_client_order_id,
        yes_price: args.yes_price,
        no_price: args.no_price,
        yes_price_dollars: args.yes_price_dollars,
        no_price_dollars: args.no_price_dollars,
        count: args.count,
        count_fp: args.count_fp,
    };
    if !args.safety.live {
        return emit_value(format, &json!({
            "dry_run": true,
            "operation": "orders.amend",
            "order_id": args.order_id,
            "payload": payload,
        }));
    }
    let response = client.amend_order(&args.order_id, &payload).await?;
    emit_value(format, &serde_json::to_value(response)?)
}

async fn run_watch_market(client: &KalshiClient, kind: WatchKind, market_ticker: Option<String>) -> Result<()> {
    client.connect_websocket().await?.watch_loop(kind, market_ticker).await
}

async fn fetch_all_markets(client: &KalshiClient, mut query: MarketsQuery, all: bool) -> Result<Vec<Market>> {
    let mut markets = Vec::new();
    loop {
        let response = client.get_markets(query.clone()).await?;
        let cursor = response.cursor.clone();
        markets.extend(response.markets);
        if !all || cursor.is_empty() {
            break;
        }
        query.cursor = Some(cursor);
    }
    Ok(markets)
}

async fn fetch_all_events(client: &KalshiClient, mut query: EventsQuery, all: bool) -> Result<Vec<Event>> {
    let mut events = Vec::new();
    loop {
        let response = client.get_events(query.clone()).await?;
        let cursor = response.cursor.clone();
        events.extend(response.events);
        if !all || cursor.is_empty() {
            break;
        }
        query.cursor = Some(cursor);
    }
    Ok(events)
}

async fn fetch_all_trades(client: &KalshiClient, mut query: TradesQuery, all: bool) -> Result<Vec<Trade>> {
    let mut trades = Vec::new();
    loop {
        let response = client.get_trades(query.clone()).await?;
        let cursor = response.cursor.clone();
        trades.extend(response.trades);
        if !all || cursor.is_empty() {
            break;
        }
        query.cursor = Some(cursor);
    }
    Ok(trades)
}

async fn fetch_all_positions(client: &KalshiClient, mut query: PositionsQuery, all: bool) -> Result<Vec<Position>> {
    let mut positions = Vec::new();
    loop {
        let response = client.get_positions(query.clone()).await?;
        let cursor = response.cursor.clone();
        positions.extend(response.market_positions);
        if !all || cursor.is_empty() {
            break;
        }
        query.cursor = Some(cursor);
    }
    Ok(positions)
}

async fn fetch_all_orders(client: &KalshiClient, mut query: OrdersQuery, all: bool) -> Result<Vec<Order>> {
    let mut orders = Vec::new();
    loop {
        let response = client.get_orders(query.clone()).await?;
        let cursor = response.cursor.clone();
        orders.extend(response.orders);
        if !all || cursor.is_empty() {
            break;
        }
        query.cursor = Some(cursor);
    }
    Ok(orders)
}

async fn fetch_all_settlements(client: &KalshiClient, mut query: SettlementsQuery, all: bool) -> Result<Vec<Settlement>> {
    let mut settlements = Vec::new();
    loop {
        let response = client.get_settlements(query.clone()).await?;
        let cursor = response.cursor.clone();
        settlements.extend(response.settlements);
        if !all || cursor.is_empty() {
            break;
        }
        query.cursor = Some(cursor);
    }
    Ok(settlements)
}

async fn fetch_all_fills(
    client: &KalshiClient,
    ticker: Option<String>,
    order_id: Option<String>,
    pagination: PaginationArgs,
    min_ts: Option<i64>,
    max_ts: Option<i64>,
    subaccount: Option<u32>,
) -> Result<Vec<Fill>> {
    let mut fills = Vec::new();
    let mut cursor = pagination.cursor;
    loop {
        let response = client
            .get_fills(
                ticker.clone(),
                order_id.clone(),
                pagination.limit,
                cursor.clone(),
                min_ts,
                max_ts,
                subaccount,
            )
            .await?;
        let next = response.cursor.clone();
        fills.extend(response.fills);
        if !pagination.all || next.is_empty() {
            break;
        }
        cursor = Some(next);
    }
    Ok(fills)
}

async fn search_markets(
    client: &KalshiClient,
    query: &str,
    status: Option<String>,
    pagination: PaginationArgs,
) -> Result<Vec<Market>> {
    let mut seen = HashSet::new();
    let mut results = Vec::new();
    let search_pages = if pagination.all { usize::MAX } else { 12 };

    let series_response = client
        .get_series_list(SeriesQuery {
            category: None,
            tags: None,
            include_volume: false,
            include_product_metadata: true,
            min_updated_ts: None,
        })
        .await?;

    for series in series_response.series {
        if !series.matches(query) {
            continue;
        }

        let series_markets = fetch_all_markets(
            client,
            MarketsQuery {
                limit: Some(100),
                cursor: None,
                event_ticker: None,
                series_ticker: Some(series.ticker),
                min_created_ts: None,
                max_created_ts: None,
                min_updated_ts: None,
                min_close_ts: None,
                max_close_ts: None,
                min_settled_ts: None,
                max_settled_ts: None,
                status: status.clone(),
                tickers: None,
                mve_filter: None,
            },
            true,
        )
        .await?;

        for market in series_markets {
            if seen.insert(market.ticker.clone()) {
                results.push(market);
                if results.len() >= pagination.limit as usize {
                    return Ok(results);
                }
            }
        }
    }

    let mut market_query = MarketsQuery {
        limit: Some(100),
        cursor: pagination.cursor.clone(),
        event_ticker: None,
        series_ticker: None,
        min_created_ts: None,
        max_created_ts: None,
        min_updated_ts: None,
        min_close_ts: None,
        max_close_ts: None,
        min_settled_ts: None,
        max_settled_ts: None,
        status: status.clone(),
        tickers: None,
        mve_filter: None,
    };

    let mut pages = 0usize;
    loop {
        let response = client.get_markets(market_query.clone()).await?;
        let next = response.cursor.clone();
        for market in response.markets {
            if market.matches(query) && seen.insert(market.ticker.clone()) {
                results.push(market);
                if results.len() >= pagination.limit as usize {
                    return Ok(results);
                }
            }
        }

        pages += 1;
        if next.is_empty() || pages >= search_pages {
            break;
        }
        market_query.cursor = Some(next);
    }

    let mut event_query = EventsQuery {
        limit: 50,
        cursor: None,
        status: status.clone(),
        series_ticker: None,
        with_nested_markets: false,
        with_milestones: false,
        min_updated_ts: None,
        min_close_ts: None,
    };

    let mut event_pages = 0usize;
    loop {
        let response = client.get_events(event_query.clone()).await?;
        let next = response.cursor.clone();
        for event in response.events {
            if event.matches(query) {
                let event_ticker = event.event_ticker.clone();
                let event_markets = fetch_all_markets(
                    client,
                    MarketsQuery {
                        limit: Some(100),
                        cursor: None,
                        event_ticker: Some(event_ticker),
                        series_ticker: None,
                        min_created_ts: None,
                        max_created_ts: None,
                        min_updated_ts: None,
                        min_close_ts: None,
                        max_close_ts: None,
                        min_settled_ts: None,
                        max_settled_ts: None,
                        status: status.clone(),
                        tickers: None,
                        mve_filter: None,
                    },
                    true,
                )
                .await?;
                for market in event_markets {
                    if seen.insert(market.ticker.clone()) {
                        results.push(market);
                        if results.len() >= pagination.limit as usize {
                            return Ok(results);
                        }
                    }
                }
            }
        }

        event_pages += 1;
        if next.is_empty() || event_pages >= search_pages {
            break;
        }
        event_query.cursor = Some(next);
    }

    Ok(results)
}

async fn watch_recent_open(
    client: &KalshiClient,
    format: OutputFormat,
    interval_seconds: u64,
    limit: u32,
    once: bool,
    query: Option<&str>,
) -> Result<()> {
    let mut seen = HashSet::new();
    loop {
        let response = client.get_markets(MarketsQuery {
            limit: Some(limit),
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
            status: Some("open".to_string()),
            tickers: None,
            mve_filter: None,
        }).await?;

        let newly_seen: Vec<Market> = response
            .markets
            .into_iter()
            .filter(|market| seen.insert(market.ticker.clone()))
            .filter(|market| market_matches_query(market, query))
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

fn filter_markets_by_query(markets: Vec<Market>, query: Option<&str>) -> Vec<Market> {
    markets
        .into_iter()
        .filter(|market| market_matches_query(market, query))
        .collect()
}

fn market_matches_query(market: &Market, query: Option<&str>) -> bool {
    query.is_none_or(|query| market.matches(query))
}

fn emit_markets(format: OutputFormat, markets: &[Market]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_markets(markets),
        OutputFormat::Json => print_json(markets),
        OutputFormat::Ndjson => print_ndjson_values(markets),
        OutputFormat::Csv => print_csv_records(markets),
    }
}

fn emit_events(format: OutputFormat, events: &[Event]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_events(events),
        OutputFormat::Json => print_json(events),
        OutputFormat::Ndjson => print_ndjson_values(events),
        OutputFormat::Csv => print_csv_records(events),
    }
}

fn emit_serieses(format: OutputFormat, series: &[Series]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_series(series),
        OutputFormat::Json => print_json(series),
        OutputFormat::Ndjson => print_ndjson_values(series),
        OutputFormat::Csv => print_csv_records(series),
    }
}

fn emit_trades(format: OutputFormat, trades: &[Trade]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_trades(trades),
        OutputFormat::Json => print_json(trades),
        OutputFormat::Ndjson => print_ndjson_values(trades),
        OutputFormat::Csv => print_csv_records(trades),
    }
}

fn emit_fills(format: OutputFormat, fills: &[Fill]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_fills(fills),
        OutputFormat::Json => print_json(fills),
        OutputFormat::Ndjson => print_ndjson_values(fills),
        OutputFormat::Csv => print_csv_records(fills),
    }
}

fn emit_positions(format: OutputFormat, positions: &[Position]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_positions(positions),
        OutputFormat::Json => print_json(positions),
        OutputFormat::Ndjson => print_ndjson_values(positions),
        OutputFormat::Csv => print_csv_records(positions),
    }
}

fn emit_settlements(format: OutputFormat, settlements: &[Settlement]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_settlements(settlements),
        OutputFormat::Json => print_json(settlements),
        OutputFormat::Ndjson => print_ndjson_values(settlements),
        OutputFormat::Csv => print_csv_records(settlements),
    }
}

fn emit_orders(format: OutputFormat, orders: &[Order]) -> Result<()> {
    match format {
        OutputFormat::Table => print_table_orders(orders),
        OutputFormat::Json => print_json(orders),
        OutputFormat::Ndjson => print_ndjson_values(orders),
        OutputFormat::Csv => print_csv_records(orders),
    }
}

fn emit_value(format: OutputFormat, value: &Value) -> Result<()> {
    match format {
        OutputFormat::Table | OutputFormat::Json => print_json(value),
        OutputFormat::Ndjson => {
            if let Some(items) = value.as_array() {
                print_ndjson_values(items)
            } else {
                print_json(value)
            }
        }
        OutputFormat::Csv => print_value_csv(value),
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
