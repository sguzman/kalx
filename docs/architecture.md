# Architecture

`kalx` is split into:

- `cli`: clap subcommands and argument shapes
- `config`: `.env`, config file precedence, profile selection, validation
- `kalshi/auth`: key loading and Kalshi signing
- `kalshi/rest`: public and authenticated REST client
- `kalshi/ws`: authenticated WebSocket connection and subscriptions
- `kalshi/models`: typed request and response models
- `output`: table/json/ndjson/csv rendering
- `logging`: tracing subscriber setup

Auth signing uses:

`timestamp + HTTP_METHOD + path_without_query`

Behavioral principles:

- read commands are safe by default
- write commands preview unless `--live` is given
- prod writes require explicit confirmation
- `api get` remains the escape hatch for uncovered endpoints

