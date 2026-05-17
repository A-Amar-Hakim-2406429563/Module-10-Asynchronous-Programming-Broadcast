use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};

use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {

    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {

            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        let text = msg.as_text().unwrap();

                        println!("From client {addr}: {text}");

                        bcast_tx.send(format!("{addr}: {text}"))?;
                    }

                    _ => break,
                }
            }

            Ok(msg) = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg)).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:2000").await?;

    println!("listening on port 2000");

    loop {

        let (socket, addr) = listener.accept().await?;

        println!("New connection from Amar's Computer {addr:?}");

        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {

            let (_req, ws_stream) =
                ServerBuilder::new()
                    .accept(socket)
                    .await
                    .unwrap();

            handle_connection(addr, ws_stream, bcast_tx)
                .await
                .unwrap();
        });
    }
}