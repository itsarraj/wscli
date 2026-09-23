use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// WebSocket URL (e.g., ws://localhost:8080 or wss://echo.websocket.org)
    url: String,

    /// Initial text payload to send upon connection
    #[arg(short, long)]
    payload: Option<String>,

    /// Exit after receiving the first message
    #[arg(short = '1', long)]
    one_shot: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    eprintln!("Connecting to {} ...", args.url);
    let (ws_stream, _) = connect_async(&args.url)
        .await
        .context("Failed to connect")?;
    eprintln!("Connected.");

    let (mut write, mut read) = ws_stream.split();

    if let Some(payload) = args.payload {
        write
            .send(Message::Text(payload))
            .await
            .context("Failed to send initial payload")?;
    }

    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        println!("{}", text);
                        if args.one_shot {
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        eprintln!("[Received {} bytes of binary data]", bin.len());
                        if args.one_shot {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(_))) => {
                        eprintln!("[Ping]");
                    }
                    Some(Ok(Message::Pong(_))) => {
                        eprintln!("[Pong]");
                    }
                    Some(Ok(Message::Close(c))) => {
                        if let Some(cf) = c {
                            eprintln!("[Connection closed: {} - {}]", cf.code, cf.reason);
                        } else {
                            eprintln!("[Connection closed]");
                        }
                        break;
                    }
                    Some(Ok(Message::Frame(_))) => {}
                    Some(Err(e)) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                    None => {
                        eprintln!("[Connection closed by server]");
                        break;
                    }
                }
            }
            line = stdin_reader.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if let Err(e) = write.send(Message::Text(text)).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                    Ok(None) => {
                        // EOF on stdin, we can break or just keep reading WS
                    }
                    Err(e) => {
                        eprintln!("Error reading stdin: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
