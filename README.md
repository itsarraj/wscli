# wscli

A headless WebSocket client CLI to connect to `ws://` or `wss://` endpoints, send payloads, and print responses (like `curl` but for WebSockets).

## Status

**built, untested**: the WebSocket client logic is built using `tokio-tungstenite`.

## Installation

```sh
cargo install --path .
```

## Usage

```sh
# Connect and interactively send/receive messages
wscli wss://echo.websocket.org

# Send a one-off payload and exit after the first response
wscli wss://echo.websocket.org -p "hello" -1
```
