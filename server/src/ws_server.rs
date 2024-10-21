use futures::{channel::mpsc::unbounded, StreamExt};
use rosc::{OscMessage, OscPacket, OscType};
use serde_json::Value;
use std::{
    fs,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

use crate::clients::{Clients, Tx};

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

        let dir = fs::read_dir("./recordings").unwrap();
        let files: Vec<OscType> = dir
            .map(|dir| OscType::String(dir.unwrap().file_name().to_str().unwrap().to_string()))
            .collect();

        let osc_msg = OscMessage {
            addr: "/recordings".to_string(),
            args: files,
        };

        let osc_packet = OscPacket::Message(osc_msg);
        let raw = rosc::encoder::encode(&osc_packet).unwrap();

        clients.send_to_client(addr, Message::Binary(raw));
    }

    let forward_task = async move {
        rx.map(Ok).forward(outgoing).await.unwrap_or_else(|e| {
            println!("Error forwarding messages to WebSocket: {:?}", e);
        });
    };

    let receive_task = async move {
        incoming
            .for_each(|message| async {
                match message {
                    Ok(msg) => match msg {
                        Message::Text(msg) => handle_client_message(msg, tx.clone()),
                        _ => println!("received unsupported message type"),
                    },
                    Err(e) => {
                        println!("Error receiving a message from {}: {:?}", addr, e);
                    }
                }
            })
            .await;
    };

    tokio::select! {
        _ = forward_task => {
            println!("Forwarding task completed for client {}", addr);
        },
        _ = receive_task => {
            println!("Receiving task completed for client {}", addr);
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

// 1729454117578
// 1729454117581
