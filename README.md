# kalx

`kalx` is a Rust CLI for exploring Kalshi's Trade API from the terminal.

It is built around:

- `clap` subcommands
- `tracing`-based logging
- `.env`-driven secret discovery
- signed REST and WebSocket auth support
- scriptable output modes

## Setup

1. Create a Kalshi API key and download the private key.
2. Copy `.env.example` to `.env`.
3. Fill in:

```dotenv
KALSHI_ENV=demo
KALSHI_API_KEY_ID=...
KALSHI_PRIVATE_KEY_PATH=/absolute/path/to/your.key
KALX_LOG=info
KALX_OUTPUT=table
```

4. Build:

```bash
cargo build
```

## Examples

```bash
kalx doctor
kalx markets list --status open --limit 25
kalx markets search inflation --status open
kalx markets recent-open --minutes 60
kalx markets watch-open --interval-seconds 15
kalx markets orderbook KXHIGHNY-24JAN01-T60
kalx events list --status open --limit 50
kalx series list --category Economics
kalx portfolio balance
kalx auth check
kalx api get /markets?limit=5
```

## Notes

- Secrets live in `.env` and key files. Both are gitignored.
- Public market-data endpoints can work without auth.
- Private portfolio and order endpoints require `KALSHI_API_KEY_ID` and `KALSHI_PRIVATE_KEY_PATH`.
- `markets search` is local substring filtering over fetched market metadata.

