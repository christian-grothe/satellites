use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::StreamExt;
use rosc::{OscMessage, OscPacket, OscType};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio_tungstenite::tungstenite::Message;

type Tx = UnboundedSender<Message>;
type ClientMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct IncomingMessage {
    message_type: String,
    data: Value,
}

#[derive(Default)]
struct Clients {
    client_map: ClientMap,
    client_array: Vec<SocketAddr>,
    current_client_index: usize,
}

impl Clients {
    fn add_client(&mut self, addr: SocketAddr, tx: Tx) {
        self.client_map.lock().unwrap().insert(addr, tx.clone());
        self.client_array.push(addr);
    }

    fn remove_client(&mut self, addr: SocketAddr) {
        self.client_map.lock().unwrap().remove(&addr);
        self.client_array.retain(|&x| x != addr);
        self.current_client_index = 0;
    }

    fn get_next_client(&mut self) -> Option<Tx> {
        if self.client_array.is_empty() {
            return None;
        }
        let client = self.client_array[self.current_client_index];
        self.current_client_index = (self.current_client_index + 1) % self.client_array.len();
        let binding = self.client_map.lock().unwrap();
        let tx = binding.get(&client);
        tx.cloned()
    }

    fn send_to_next_client(&mut self, msg: Message) {
        if let Some(client) = self.get_next_client() {
            if let Err(e) = client.unbounded_send(msg) {
                println!("Error sending message to client: {:?}", e);
            }
        }
    }

    fn broadcast(&self, msg: Message) {
        let binding = self.client_map.lock().unwrap();
        for (_, tx) in binding.iter() {
            if let Err(e) = tx.unbounded_send(msg.clone()) {
                println!("Error sending message to client: {:?}", e);
            }
        }
    }
}

struct OscServer {
    socket: UdpSocket,
    buf: Vec<u8>,
    msg: Option<(usize, SocketAddr)>,
    clients: Arc<Mutex<Clients>>,
}

impl OscServer {
    async fn run(mut self) -> Result<(), io::Error> {
        loop {
            if let Some((size, _)) = self.msg {
                let (_, packet) = rosc::decoder::decode_udp(&self.buf[..size]).unwrap();

                match packet.clone() {
                    rosc::OscPacket::Message(mut msg) => {
                        let packet_with_timestamp = self.add_timestamp(&mut msg);
                        self.handle_message(&msg, &packet_with_timestamp);
                    }

                    rosc::OscPacket::Bundle(bundle) => {
                        println!("got osc bundle: {:?}", bundle);
                    }
                }
            }

            self.msg = Some(self.socket.recv_from(&mut self.buf).await?);
        }
    }

    fn handle_message(&self, msg: &rosc::OscMessage, packet: &OscPacket) {
        match msg.addr.as_str() {
            "/sampler/play/next" => {
                let raw = rosc::encoder::encode(packet).unwrap();
                self.clients
                    .lock()
                    .unwrap()
                    .send_to_next_client(Message::Binary(raw));
            }
            "/sampler/play" => {
                let raw = rosc::encoder::encode(packet).unwrap();
                self.clients.lock().unwrap().broadcast(Message::Binary(raw));
            }
            _ => println!("unknown message"),
        }
    }

    fn add_timestamp(&self, msg: &mut rosc::OscMessage) -> OscPacket {
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis() + 1000;

        msg.args
            .push(rosc::OscType::String("timestamp".to_string()));
        msg.args.push(rosc::OscType::Long(timestamp as i64));

        OscPacket::Message(msg.clone())
    }
}

async fn handle_websocket_connection(
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
        if let Some(timestamp) = incoming_message.data.as_i64() {
            let current_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let osc_message: OscMessage = OscMessage {
                addr: "/sync".to_string(),
                args: vec![
                    OscType::String("t1".to_string()),
                    OscType::Long(timestamp),
                    OscType::String("t2".to_string()),
                    OscType::Long(current_timestamp as i64),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clients_st = Arc::new(Mutex::new(Clients::default()));

    let socket_udp = UdpSocket::bind("127.0.0.1:8081").await?;
    let socket_tcp = TcpListener::bind("127.0.0.1:8080").await?;

    let osc_server = OscServer {
        socket: socket_udp,
        buf: vec![0; 1024],
        msg: None,
        clients: clients_st.clone(),
    };

    let udp_handler = tokio::spawn(async move {
        let _ = osc_server.run().await;
    });

    let tcp_handler = tokio::spawn(async move {
        while let Ok((stream, addr)) = socket_tcp.accept().await {
            tokio::spawn(handle_websocket_connection(
                clients_st.clone(),
                stream,
                addr,
            ));
        }
    });

    udp_handler.await.unwrap();
    tcp_handler.await.unwrap();

    Ok(())
}
