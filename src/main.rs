use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use sled::{Db, IVec};
use serde_json::Value;
use anyhow::Result;
use futures_util::stream::StreamExt;
#[tokio::main]
async fn main() -> Result<()> {
    let db: Db = sled::open("nostr_relay_db")?;  // Open the database

    let listener = TcpListener::bind("192.168.1.78:8080").await?;  // Setup TCP listener
    println!("Listening on: 192.168.1.78");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, db.clone()));
    }

    Ok(())
}

async fn handle_connection(raw_stream: tokio::net::TcpStream, db: Db) -> Result<()> {
    let ws_stream = accept_async(raw_stream).await?;
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg? {
            tokio_tungstenite::tungstenite::Message::Text(txt) => {
                if let Ok(event) = serde_json::from_str::<Value>(&txt) {
                    // Store event in sled
                    let event_id = event["id"].as_str().unwrap_or_default();
                    db.insert(event_id, txt.as_bytes())?;
                    // Optionally broadcast this event to other subscribers
                }
            },
            _ => {}  // Handle other message types if necessary
        }
    }

    Ok(())
}
