use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use rosc::OscPacket;
use tokio::net::UdpSocket;
use tokio_tungstenite::tungstenite::Message;

use crate::clients::Clients;

pub struct OscServer {
    pub socket: UdpSocket,
    pub buf: Vec<u8>,
    pub msg: Option<(usize, SocketAddr)>,
    pub clients: Arc<Mutex<Clients>>,
}

impl OscServer {
    pub async fn run(mut self) -> Result<(), io::Error> {
        loop {
            if let Some((size, _)) = self.msg {
                let (_, packet) = rosc::decoder::decode_udp(&self.buf[..size]).unwrap();

                match packet {
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

    pub fn handle_message(&self, msg: &rosc::OscMessage, packet: &OscPacket) {
        match msg.addr.as_str() {
            "/sampler/play/next" => {
                let raw = rosc::encoder::encode(packet).unwrap();
                self.clients
                    .lock()
                    .unwrap()
                    .send_to_next_client(Message::Binary(raw));
            }
            "/sampler/play/rand" => {
                let raw = rosc::encoder::encode(packet).unwrap();
                self.clients
                    .lock()
                    .unwrap()
                    .send_to_random_client(Message::Binary(raw));
            }
            "/sampler/play" => {
                let raw = rosc::encoder::encode(packet).unwrap();
                self.clients.lock().unwrap().broadcast(Message::Binary(raw));
            }
            _ => {
                let raw = rosc::encoder::encode(packet).unwrap();
                self.clients.lock().unwrap().broadcast(Message::Binary(raw));
            }
        }
    }

    pub fn add_timestamp(&self, msg: &mut rosc::OscMessage) -> OscPacket {
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis() + 1000;

        msg.args
            .push(rosc::OscType::String("timestamp".to_string()));
        msg.args.push(rosc::OscType::String(timestamp.to_string()));

        OscPacket::Message(msg.clone())
    }
}
