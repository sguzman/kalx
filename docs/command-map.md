# Command Map

## Implemented typed commands

- `doctor`
- `config show|path|init|validate`
- `auth check|keys`
- `exchange status|schedule`
- `series list|get`
- `events list|get|markets`
- `markets search|list|get|candles|orderbook|trades|watch|recent-open|watch-open`
- `portfolio balance|positions|fills|settlements`
- `orders list|get|create|cancel|cancel-market|amend`
- `watch market|orderbook|fills|positions`
- `export markets|trades|positions|fills`
- `api get`
- `completions fish|bash|zsh|powershell`

## Notes

- `auth whoami` is omitted because there is no dedicated current-user Trade API endpoint in the current CLI implementation.
- `api get --auth` is the fallback for any documented endpoint not yet mapped to a typed subcommand.
- The `markets watch` command is an alias-shaped typed command that routes to the WebSocket market watch flow.

