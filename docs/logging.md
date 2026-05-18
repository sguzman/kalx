# Logging

`kalx` uses `tracing`.

Controls:

- `KALX_LOG=info|debug|trace`
- `-v`, `-vv`
- `--log-json`

Defaults:

- compact human-readable stderr logs
- `info` level unless overridden

Logging policy:

- request metadata may be logged
- API key values are not printed in full
- private key contents are never logged
- full signatures are never logged

