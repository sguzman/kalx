//! Local stdio MCP adapter. Only allowlisted public GETs, no credentials.
use std::io::{self, BufRead, Write};
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use kalx::{config::AppConfig, kalshi::KalshiClient};
use reqwest::Method;
use serde_json::{Value, json};

const PROTOCOL: &str = "2025-11-25";

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name, "description": description,
        "inputSchema": {"type": "object", "properties": properties,
            "required": required, "additionalProperties": false},
        "annotations": {"readOnlyHint": true, "destructiveHint": false,
            "idempotentHint": true, "openWorldHint": true}
    })
}

fn tools() -> Vec<Value> {
    let ticker = json!({"type":"string","minLength":1,"maxLength":200,
        "pattern":"^[A-Za-z0-9_.-]+$"});
    let status = json!({"type":"string","description":"Kalshi market/event status; defaults to open"});
    let cursor = json!({"type":"string","description":"Opaque cursor from the previous response"});
    let limit = json!({"type":"integer","minimum":1,"maximum":100,"default":25});
    vec![
        tool("list_markets", "List real Kalshi markets with optional status, event and series filters. Returns a pagination cursor.", json!({
            "status":status,"event_ticker":ticker,"series_ticker":ticker,"cursor":cursor,"limit":limit
        }), &[]),
        tool("search_markets", "Search market titles, tickers and subtitles in a bounded scan of up to five pages. Continue with the returned cursor; no matches does not imply global absence.", json!({
            "query":{"type":"string","minLength":1,"maxLength":200},
            "status":status,"event_ticker":ticker,"series_ticker":ticker,"cursor":cursor,
            "pages":{"type":"integer","minimum":1,"maximum":5,"default":3}
        }), &["query"]),
        tool("get_market", "Get a single market, including prices, volume, close time and settlement rules.", json!({"ticker":ticker}), &["ticker"]),
        tool("list_events", "Browse events with cursor pagination.", json!({
            "status":status,"series_ticker":ticker,"cursor":cursor,"limit":limit
        }), &[]),
        tool("get_event", "Get an event and its nested markets.", json!({"ticker":ticker}), &["ticker"]),
        tool("search_series", "Find series by title, ticker, category or tags. Use offset to continue through matches.", json!({
            "query":{"type":"string","maxLength":200},
            "category":{"type":"string","maxLength":200},"limit":limit,
            "offset":{"type":"integer","minimum":0,"maximum":100000,"default":0}
        }), &[]),
        tool("get_series", "Get series metadata, including settlement sources when provided.", json!({"ticker":ticker}), &["ticker"]),
        tool("get_orderbook", "Read public bid depth for a market.", json!({
            "ticker":ticker,"depth":{"type":"integer","minimum":1,"maximum":100,"default":10}
        }), &["ticker"]),
        tool("list_trades", "List recent public trades for a market. Returns a cursor.", json!({
            "ticker":ticker,"limit":limit,"cursor":cursor
        }), &["ticker"]),
        tool("get_exchange_status", "Get public exchange and trading availability status.", json!({}), &[])
    ]
}

fn optional_str<'a>(args: &'a Value, key: &str, max: usize) -> Result<Option<&'a str>> {
    match args.get(key) {
        None => Ok(None),
        Some(v) => {
            let s = v.as_str().with_context(|| format!("{key} must be a string"))?;
            ensure!(s.len() <= max && !s.trim().is_empty(), "invalid {key}");
            Ok(Some(s))
        }
    }
}

fn ticker<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>> {
    let value = optional_str(args, key, 200)?;
    if let Some(s) = value {
        ensure!(s != "." && s != ".." && s.bytes().all(|b|
            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.')), "invalid ticker");
    }
    Ok(value)
}

fn required_ticker<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    ticker(args, key)?.with_context(|| format!("missing {key}"))
}

fn number(args: &Value, key: &str, default: u64, min: u64, max: u64) -> Result<u64> {
    let value = match args.get(key) {
        None => default,
        Some(v) => v.as_u64().with_context(|| format!("{key} must be an integer"))?,
    };
    ensure!((min..=max).contains(&value), "{key} must be between {min} and {max}");
    Ok(value)
}

fn filters(args: &Value, event: bool) -> Result<Vec<(String, String)>> {
    let mut query = vec![("status".into(), optional_str(args, "status", 40)?.unwrap_or("open").into())];
    for key in if event { &["series_ticker"][..] } else { &["event_ticker", "series_ticker"][..] } {
        if let Some(s) = ticker(args, key)? {
            query.push(((*key).into(), s.into()));
        }
    }
    Ok(query)
}

fn add_cursor(query: &mut Vec<(String, String)>, args: &Value) -> Result<()> {
    if let Some(s) = optional_str(args, "cursor", 4096)? {
        query.push(("cursor".into(), s.into()));
    }
    Ok(())
}

fn matches(value: &Value, fields: &[&str], query: &str) -> bool {
    let haystack = fields.iter().map(|f| match value.get(*f) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" "),
        _ => String::new(),
    }).collect::<Vec<_>>().join(" ").to_lowercase();
    query.split_whitespace().all(|word| haystack.contains(&word.to_lowercase()))
}

async fn get(client: &KalshiClient, path: &str, query: &[(String, String)]) -> Result<Value> {
    client.public_json(Method::GET, path, query, None).await
}

async fn call(client: &KalshiClient, name: &str, args: &Value) -> Result<Value> {
    let definitions = tools();
    let definition = definitions.iter().find(|t| t["name"].as_str() == Some(name))
        .context("unknown tool")?;
    let obj = args.as_object().context("arguments must be an object")?;
    for key in obj.keys() {
        ensure!(definition["inputSchema"]["properties"].get(key).is_some(), "unknown argument: {key}");
    }
    match name {
        "list_markets" | "list_events" => {
            let events = name == "list_events";
            let mut query = filters(args, events)?;
            add_cursor(&mut query, args)?;
            query.push(("limit".into(), number(args,"limit",25,1,100)?.to_string()));
            get(client, if events {"/events"} else {"/markets"}, &query).await
        }
        "search_markets" => {
            let needle = optional_str(args,"query",200)?.context("missing query")?;
            let pages = number(args,"pages",3,1,5)?;
            let mut cursor = optional_str(args,"cursor",4096)?.unwrap_or("").to_owned();
            let mut found = Vec::new();
            let mut scanned = 0;
            let mut scanned_pages = 0;
            for _ in 0..pages {
                let mut query = filters(args, false)?;
                query.push(("limit".into(),"40".into()));
                if !cursor.is_empty() { query.push(("cursor".into(),cursor.clone())); }
                let response = get(client,"/markets",&query).await?;
                let markets = response["markets"].as_array().context("missing markets array")?;
                scanned += markets.len();
                scanned_pages += 1;
                for m in markets {
                    if matches(m,&["ticker","event_ticker","title","subtitle","yes_sub_title","no_sub_title"],needle) {
                        found.push(json!({
                            "ticker":m["ticker"],"event_ticker":m["event_ticker"],"title":m["title"],
                            "subtitle":m["subtitle"],"yes_sub_title":m["yes_sub_title"],"status":m["status"],
                            "yes_bid_dollars":m["yes_bid_dollars"],"yes_ask_dollars":m["yes_ask_dollars"],
                            "last_price_dollars":m["last_price_dollars"],"close_time":m["close_time"],
                            "volume_fp":m["volume_fp"],"volume_24h_fp":m["volume_24h_fp"]
                        }));
                    }
                }
                let next = response["cursor"].as_str().unwrap_or("").to_owned();
                ensure!(next.is_empty() || next != cursor, "upstream returned a repeated cursor");
                cursor = next;
                if cursor.is_empty() { break; }
            }
            Ok(json!({"query":needle,"markets":found,"scanned":scanned,"pages_scanned":scanned_pages,
                "cursor":cursor,"complete":cursor.is_empty()}))
        }
        "get_market" => get(client,&format!("/markets/{}",required_ticker(args,"ticker")?),&[]).await,
        "get_event" => get(client,&format!("/events/{}",required_ticker(args,"ticker")?),
            &[("with_nested_markets".into(),"true".into())]).await,
        "get_series" => get(client,&format!("/series/{}",required_ticker(args,"ticker")?),&[]).await,
        "search_series" => {
            let mut query = Vec::new();
            if let Some(s) = optional_str(args,"category",200)? { query.push(("category".into(),s.into())); }
            let needle = optional_str(args,"query",200)?.unwrap_or("");
            let limit = number(args,"limit",25,1,100)? as usize;
            let offset = number(args,"offset",0,0,100000)? as usize;
            let response = get(client,"/series",&query).await?;
            let all = response["series"].as_array().context("missing series array")?;
            let filtered: Vec<_> = all.iter().filter(|s| matches(s,&["ticker","title","category","tags"],needle)).collect();
            let total = filtered.len();
            let series: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();
            let next = (offset + series.len() < total).then_some(offset + series.len());
            Ok(json!({"series":series,"total_matches":total,"next_offset":next}))
        }
        "get_orderbook" => get(client,&format!("/markets/{}/orderbook",required_ticker(args,"ticker")?),
            &[("depth".into(),number(args,"depth",10,1,100)?.to_string())]).await,
        "list_trades" => {
            let mut query = vec![("ticker".into(),required_ticker(args,"ticker")?.into()),
                ("limit".into(),number(args,"limit",25,1,100)?.to_string())];
            add_cursor(&mut query,args)?;
            get(client,"/markets/trades",&query).await
        }
        "get_exchange_status" => get(client,"/exchange/status",&[]).await,
        _ => bail!("unknown tool"),
    }
}

fn rpc_error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

async fn handle(client: &KalshiClient, req: Value) -> Option<Value> {
    let id = req.get("id").cloned();
    if req["jsonrpc"] != "2.0" || !req["method"].is_string() {
        return Some(rpc_error(id.unwrap_or(Value::Null),-32600,"Invalid Request"));
    }
    let id = id?; // notifications have no response
    let params = &req["params"];
    let result = match req["method"].as_str().unwrap_or("") {
        "initialize" => {
            let requested = params["protocolVersion"].as_str().unwrap_or(PROTOCOL);
            let version = match requested {
                "2024-11-05" | "2025-03-26" | "2025-06-18" | PROTOCOL => requested,
                _ => PROTOCOL,
            };
            json!({"protocolVersion":version,"capabilities":{"tools":{"listChanged":false}},
                "serverInfo":{"name":"kalx","version":env!("CARGO_PKG_VERSION")},
                "instructions":"Read-only production Kalshi market data. Search scans bounded pages; follow cursors. No authentication or trading tools."})
        }
        "ping" => json!({}),
        "tools/list" => json!({"tools":tools()}),
        "tools/call" => {
            let Some(name) = params["name"].as_str() else {
                return Some(rpc_error(id,-32602,"Missing tool name"));
            };
            let empty = json!({});
            let args = params.get("arguments").unwrap_or(&empty);
            let outcome = tokio::time::timeout(Duration::from_secs(45),call(client,name,args)).await;
            let (value, failed) = match outcome {
                Ok(Ok(value)) => (value,false),
                Ok(Err(err)) => (json!({"error":err.to_string()}),true),
                Err(_) => (json!({"error":"Kalshi request timed out"}),true),
            };
            json!({"content":[{"type":"text","text":value.to_string()}],"isError":failed})
        }
        _ => return Some(rpc_error(id,-32601,"Method not found")),
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Deliberately do not call AppConfig::load: no dotenv, user config or private key.
    let mut config = AppConfig::defaults();
    config.profile = "prod".into();
    let client = KalshiClient::new(config)?;
    let mut out = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let response = match serde_json::from_str(&line) {
            Ok(request) => handle(&client,request).await,
            Err(_) => Some(rpc_error(Value::Null,-32700,"Parse error")),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut out,&response)?;
            writeln!(out)?;
            out.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method,path,query_param}};

    #[tokio::test]
    async fn discovery_and_notifications() {
        let client = KalshiClient::new(AppConfig::defaults()).unwrap();
        let result = handle(&client,json!({"jsonrpc":"2.0","id":1,"method":"tools/list"})).await.unwrap();
        let listed = result["result"]["tools"].as_array().unwrap();
        assert_eq!(listed.len(),10);
        assert!(listed.iter().all(|t| t["annotations"]["readOnlyHint"] == true));
        assert!(handle(&client,json!({"jsonrpc":"2.0","method":"notifications/initialized"})).await.is_none());
        assert!(call(&client,"create_order",&json!({})).await.is_err());
        assert!(call(&client,"get_market",&json!({"ticker":"../portfolio/balance"})).await.is_err());
        assert!(call(&client,"list_markets",&json!({"limit":1000})).await.is_err());
    }

    #[tokio::test]
    async fn search_continues_with_cursor() {
        let server = MockServer::start().await;
        let mut config = AppConfig::defaults();
        config.profiles.get_mut("demo").unwrap().rest_base_url = server.uri();
        let client = KalshiClient::new(config).unwrap();
        Mock::given(method("GET")).and(path("/markets")).and(query_param("limit","40"))
            .and(query_param("cursor","page2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "markets":[{"ticker":"B","title":"Mexico inflation"}],"cursor":""
            }))).expect(1).mount(&server).await;
        Mock::given(method("GET")).and(path("/markets")).and(query_param("limit","40"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "markets":[{"ticker":"A","title":"US inflation"}],"cursor":"page2"
            }))).expect(1).mount(&server).await;
        let result = call(&client,"search_markets",&json!({"query":"Mexico inflation","pages":2})).await.unwrap();
        assert_eq!(result["markets"][0]["ticker"],"B");
        assert_eq!(result["scanned"],2);
        assert_eq!(result["complete"],true);
    }
}
