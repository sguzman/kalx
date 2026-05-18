# Architecture

`kalx` is split into:

- `cli`: command and argument parsing
- `config`: `.env`, config file, and profile resolution
- `kalshi`: signed/public API client and typed response models
- `output`: table/json/ndjson rendering
- `logging`: tracing subscriber setup

The client signs authenticated requests using Kalshi's documented payload:

`timestamp + HTTP_METHOD + path_without_query`

