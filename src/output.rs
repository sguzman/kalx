use anyhow::Result;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use serde::Serialize;

use crate::kalshi::{
    BalanceResponse, Event, Fill, Market, MarketOrderbookResponse, Order, Position, Series, Trade,
};

pub fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_ndjson_values<T: Serialize>(values: &[T]) -> Result<()> {
    for value in values {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

pub fn print_table_markets(markets: &[Market]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["ticker", "status", "title", "yes bid", "yes ask", "volume", "open time"]);
    for market in markets {
        table.add_row([
            market.ticker.as_str(),
            market.status.as_deref().unwrap_or(""),
            market.title.as_deref().unwrap_or(""),
            market.yes_bid_dollars.as_deref().unwrap_or(""),
            market.yes_ask_dollars.as_deref().unwrap_or(""),
            market.volume_fp.as_deref().unwrap_or(""),
            market.open_time.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_events(events: &[Event]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["event", "series", "title", "category", "updated"]);
    for event in events {
        table.add_row([
            event.event_ticker.as_str(),
            event.series_ticker.as_deref().unwrap_or(""),
            event.title.as_deref().unwrap_or(""),
            event.category.as_deref().unwrap_or(""),
            event.last_updated_ts.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_series(series: &[Series]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["ticker", "category", "frequency", "title", "volume"]);
    for item in series {
        table.add_row([
            item.ticker.as_str(),
            item.category.as_deref().unwrap_or(""),
            item.frequency.as_deref().unwrap_or(""),
            item.title.as_deref().unwrap_or(""),
            item.volume_fp.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_trades(trades: &[Trade]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["trade id", "ticker", "yes price", "count", "created"]);
    for trade in trades {
        table.add_row([
            trade.trade_id.as_deref().unwrap_or(""),
            trade.ticker.as_deref().unwrap_or(""),
            trade.yes_price_dollars.as_deref().unwrap_or(""),
            trade.count_fp.as_deref().unwrap_or(""),
            trade.created_time.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_fills(fills: &[Fill]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["fill id", "market", "side", "action", "price", "count", "created"]);
    for fill in fills {
        table.add_row([
            fill.fill_id.as_deref().unwrap_or(""),
            fill.market_ticker.as_deref().unwrap_or(""),
            fill.side.as_deref().unwrap_or(""),
            fill.action.as_deref().unwrap_or(""),
            fill.yes_price_dollars.as_deref().unwrap_or(""),
            fill.count_fp.as_deref().unwrap_or(""),
            fill.created_time.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_positions(positions: &[Position]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["ticker", "position", "fees paid", "market exposure"]);
    for position in positions {
        table.add_row([
            position.ticker.as_deref().unwrap_or(""),
            position.position.as_deref().unwrap_or(""),
            position.fees_paid.as_deref().unwrap_or(""),
            position.market_exposure.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_orders(orders: &[Order]) -> Result<()> {
    let mut table = new_table();
    table.set_header(["order id", "ticker", "status", "side", "action", "price", "remaining"]);
    for order in orders {
        table.add_row([
            order.order_id.as_deref().unwrap_or(""),
            order.ticker.as_deref().unwrap_or(""),
            order.status.as_deref().unwrap_or(""),
            order.side.as_deref().unwrap_or(""),
            order.action.as_deref().unwrap_or(""),
            order.yes_price_dollars.as_deref().unwrap_or(""),
            order.remaining_count_fp.as_deref().unwrap_or(""),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_table_balance(balance: &BalanceResponse) -> Result<()> {
    let mut table = new_table();
    table.set_header(["balance_cents", "portfolio_value_cents", "updated_ts"]);
    table.add_row([
        Cell::new(balance.balance),
        Cell::new(balance.portfolio_value),
        Cell::new(balance.updated_ts),
    ]);
    println!("{table}");
    Ok(())
}

pub fn print_table_orderbook(ticker: &str, orderbook: &MarketOrderbookResponse) -> Result<()> {
    let mut table = new_table();
    table.set_header(["ticker", "side", "price", "count"]);

    for level in &orderbook.orderbook_fp.yes_dollars {
        table.add_row([ticker, "yes", level.0.as_str(), level.1.as_str()]);
    }
    for level in &orderbook.orderbook_fp.no_dollars {
        table.add_row([ticker, "no", level.0.as_str(), level.1.as_str()]);
    }
    println!("{table}");
    Ok(())
}

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}
