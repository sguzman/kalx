# Local MCP server

`kalx-mcp` is a separate, read-only stdio MCP server for real (production)
public Kalshi data. It never loads `.env`, user configuration or API keys.
It exposes only fixed GET routes; no order, portfolio, arbitrary API or shell
tools. No TCP listener or public service is created.

## Nix on Linux

```fish
nom build
./result/bin/kalx --help
./result/bin/kalx-mcp
```

The flake uses Crane to compile dependencies in a separate reusable Nix
artifact. Cargo.lock remains unchanged and all builds use --locked.
To build only the dependency artifact, run `nom build .#cargo-artifacts`.
The package also supplies a CA bundle to the sandbox for its HTTP tests;
normal runtime certificate verification remains enabled.

The MCP binary waits for newline-delimited JSON-RPC on stdin. Ctrl+C or EOF
stops it. The package includes both executables, and the flake also exposes
`nix run .#mcp`, `nix develop`, and `nix flake check`.

Install a persistent generation for local clients:

```fish
nix profile install github:sguzman/kalx
codex mcp add kalx -- ~/.nix-profile/bin/kalx-mcp
codex mcp list
```

If your Nix profile is not at that path, use the absolute path printed by
`command -v kalx-mcp`. For declarative setup, the equivalent Codex config is:

```toml
[mcp_servers.kalx]
command = "/home/YOUR_USER/.nix-profile/bin/kalx-mcp"
tool_timeout_sec = 60
```

The server requires no API key for public browsing. A hosted ChatGPT web
conversation cannot directly launch your local stdio server. Use a supported
remote plugin or OpenAI's Secure MCP Tunnel if web access is needed; see
https://developers.openai.com/api/docs/guides/secure-mcp-tunnels .
Do not expose a local port publicly merely to make it accessible.

## Tools

- `list_markets`, `search_markets`, `get_market`
- `list_events`, `get_event`
- `search_series`, `get_series`
- `get_orderbook`, `list_trades`, `get_exchange_status`

Search uses bounded market-page scans, not an exchange-wide text-search API.
Use a series filter or search series first, then continue using the returned
cursor. Search returns summary fields; get_market returns raw market data,
including settlement rules. List endpoints preserve upstream cursor and
payload. All data is fetched on demand.

## Smoke test

```fish
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"smoke","version":"1"}}}' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_markets","arguments":{"limit":2}}}' | ./result/bin/kalx-mcp
```

Expected: three JSON-RPC responses, ten tools, and a live markets response
(or a reported upstream error). No unsolicited output on stdout.
```fish
nom build
nix flake check
```
