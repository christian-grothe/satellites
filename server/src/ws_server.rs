use futures::{channel::mpsc::unbounded, StreamExt};
use rosc::{OscMessage, OscPacket, OscType};
use serde_json::Value;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

use crate::clients::{Clients, SendTo, Tx};

const PING_INTERVAL: Duration = Duration::from_secs(10);
const PONG_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct IncomingMessage {
    message_type: String,
    data: Value,
}

pub async fn handle_websocket_connection(
    clients: Arc<Mutex<Clients>>,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    println!("new tcp connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occured");

    println!("WebSocket Connection established");

    let (tx, rx) = unbounded();
    let (outgoing, incoming) = ws_stream.split();

    {
        let mut clients = clients.lock().unwrap();
        clients.add_client(addr, tx.clone());
        clients.send_recording_list(SendTo::Single(addr));
    }

    let forward_task = async move {
        rx.map(Ok).forward(outgoing).await.unwrap_or_else(|e| {
            println!("Error forwarding messages to WebSocket: {:?}", e);
        });
    };

    let clients_clone = clients.clone();
    let receive_task = async move {
        incoming
            .for_each(|message| async {
                let mut clients = clients_clone.lock().unwrap();
                match message {
                    Ok(msg) => match msg {
                        Message::Text(msg) => handle_client_message(msg, tx.clone()),
                        Message::Pong(..) => handle_pong_message(&mut clients, addr),
                        _ => println!("received unsupported message type"),
                    },
                    Err(e) => {
                        println!("Error receiving a message from {}: {:?}", addr, e);
                    }
                }
            })
            .await;
    };

    let clients_clone = clients.clone();
    let ping_task = async move {
        let mut interval = tokio::time::interval(PING_INTERVAL);
        loop {
            interval.tick().await;
            let mut clients = clients_clone.lock().unwrap();

            let now = std::time::Instant::now();

            let inactive_clients: Vec<SocketAddr> = clients
                .last_pong
                .iter()
                .filter(|(_, &last_pong)| now.duration_since(last_pong) > PONG_TIMEOUT)
                .map(|(&addr, _)| addr)
                .collect();

            for addr in inactive_clients {
                clients.remove_client(addr);
            }

            clients.send_to_client(addr, Message::Ping(vec![]));
        }
    };

    tokio::select! {
        _ = forward_task => {
            println!("Forwarding task completed for client {}", addr);
        },
        _ = receive_task => {
            println!("Receiving task completed for client {}", addr);
        },
        _ = ping_task => {
            println!("Ping task completed for client {}", addr);
        }
    }

    let mut clients = clients.lock().unwrap();
    clients.remove_client(addr);
    println!("Client {} disconnected", addr);
}

fn handle_client_message(msg: String, tx: Tx) {
    let incoming_message: IncomingMessage = serde_json::from_str(&msg).unwrap();

    if incoming_message.message_type == "sync" {
        if let Some(client_timestamp) = incoming_message.data.as_str() {
            let current_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            println!("Received sync message with timestamp: {}", client_timestamp);
            println!("current time is: {}", current_timestamp);

            let osc_message: OscMessage = OscMessage {
                addr: "/sync".to_string(),
                args: vec![
                    OscType::String("t1".to_string()),
                    OscType::String(client_timestamp.to_string()),
                    OscType::String("t2".to_string()),
                    OscType::String(current_timestamp.to_string()),
                ],
            };
            let osc_packet: OscPacket = OscPacket::Message(osc_message);
            let raw = rosc::encoder::encode(&osc_packet).unwrap();
            let response_message = Message::Binary(raw);

            if let Err(e) = tx.unbounded_send(response_message) {
                println!("Error sending message to client: {:?}", e);
            }
        } else {
            println!("Invalid timestamp format");
        }
    }
}

fn handle_pong_message(clients: &mut Clients, addr: SocketAddr) {
    if let Some(instant) = clients.last_pong.get_mut(&addr) {
        *instant = std::time::Instant::now();
    }
}
