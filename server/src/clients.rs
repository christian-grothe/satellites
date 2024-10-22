use futures::channel::mpsc::UnboundedSender;
use rosc::{OscMessage, OscPacket, OscType};
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio_tungstenite::tungstenite::Message;

pub type Tx = UnboundedSender<Message>;
pub type ClientMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Default)]
pub struct Clients {
    client_map: ClientMap,
    client_array: Vec<SocketAddr>,
    current_client_index: usize,
}

pub enum SendTo {
    Single(SocketAddr),
    Broadcast,
}

impl Clients {
    pub fn add_client(&mut self, addr: SocketAddr, tx: Tx) {
        self.client_map.lock().unwrap().insert(addr, tx.clone());
        self.client_array.push(addr);
    }

    pub fn remove_client(&mut self, addr: SocketAddr) {
        if self.client_map.lock().unwrap().remove(&addr).is_some() {
            self.client_array.retain(|&x| x != addr);
            self.current_client_index = 0;
        }
    }

    pub fn send_to_next_client(&mut self, msg: Message) {
        if let Some(client) = self.get_next_client() {
            Clients::handle_message_error(client.unbounded_send(msg));
        }
    }

    pub fn send_to_client(&self, addr: SocketAddr, msg: Message) {
        let binding = self.client_map.lock().unwrap();
        if let Some(tx) = binding.get(&addr) {
            Clients::handle_message_error(tx.unbounded_send(msg));
        }
    }

    pub fn send_to_random_client(&self, msg: Message) {
        if let Some(client) = self.get_random_client() {
            Clients::handle_message_error(client.unbounded_send(msg));
        }
    }

    pub fn send_recording_list(&self, send_to: SendTo) {
        let files: Vec<OscType> = fs::read_dir("./recordings")
            .unwrap()
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| OscType::String(s.to_string()))
                })
            })
            .collect();

        let osc_msg = OscMessage {
            addr: "/recordings".to_string(),
            args: files,
        };

        let osc_packet = OscPacket::Message(osc_msg);
        let raw = rosc::encoder::encode(&osc_packet).unwrap();

        match send_to {
            SendTo::Single(addr) => {
                self.send_to_client(addr, Message::Binary(raw));
            }
            SendTo::Broadcast => {
                self.broadcast(Message::Binary(raw));
            }
        }
    }

    pub fn broadcast(&self, msg: Message) {
        let binding = self.client_map.lock().unwrap();
        for (_, tx) in binding.iter() {
            if let Err(e) = tx.unbounded_send(msg.clone()) {
                println!("Error sending message to client: {:?}", e);
            }
        }
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

    fn get_random_client(&self) -> Option<Tx> {
        if self.client_array.is_empty() {
            return None;
        }
        let i: usize = rand::random::<usize>() % self.client_array.len();
        let client = self.client_array[i];
        let binding = self.client_map.lock().unwrap();
        let tx = binding.get(&client);
        tx.cloned()
    }

    fn handle_message_error<T>(result: Result<(), T>)
    where
        T: std::fmt::Debug,
    {
        if let Err(e) = result {
            println!("Error sending message to client: {:?}", e);
        }
    }
}
