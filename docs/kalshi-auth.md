# Kalshi Auth

Authenticated Kalshi REST requests require:

- `KALSHI-ACCESS-KEY`
- `KALSHI-ACCESS-TIMESTAMP`
- `KALSHI-ACCESS-SIGNATURE`

`kalx` signs:

`timestamp + HTTP_METHOD + path_without_query`

Important:

- The query string is not part of the signed path.
- `kalx` supports PKCS#8 PEM and PKCS#1 PEM private keys.
- WebSocket auth uses the same signing model against the documented WebSocket path.

Local secret model:

- `.env` stores `KALSHI_API_KEY_ID`
- `.env` stores `KALSHI_PRIVATE_KEY_PATH`
- the private key contents stay in the PEM file

