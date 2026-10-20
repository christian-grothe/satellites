use futures::channel::mpsc::UnboundedSender;
use rosc::{OscMessage, OscPacket, OscType};
use std::{collections::HashMap, fs, net::SocketAddr};
use tokio_tungstenite::tungstenite::Message;

pub type Tx = UnboundedSender<Message>;
pub type ClientMap = HashMap<SocketAddr, Tx>;

#[derive(Default)]
pub struct Clients {
    client_map: ClientMap,
    client_array: Vec<SocketAddr>,
    pub last_pong: HashMap<SocketAddr, std::time::Instant>,
    current_client_index: usize,
}

pub enum SendTo {
    Single(SocketAddr),
    Broadcast,
}

impl Clients {
    pub fn add_client(&mut self, addr: SocketAddr, tx: Tx) {
        self.client_map.insert(addr, tx.clone());
        self.client_array.push(addr);
        self.last_pong.insert(addr, std::time::Instant::now());
    }

    pub fn remove_client(&mut self, addr: SocketAddr) {
        if self.client_map.remove(&addr).is_some() {
            self.client_array.retain(|&x| x != addr);
            self.last_pong.remove(&addr);
        }
    }

    pub fn send_to_next_client(&mut self, msg: Message) {
        if let Some(client) = self.get_next_client() {
            Clients::handle_message_error(client.unbounded_send(msg));
        }
    }

    pub fn send_to_client(&self, addr: SocketAddr, msg: Message) {
        if let Some(tx) = self.client_map.get(&addr) {
            Clients::handle_message_error(tx.unbounded_send(msg));
        }
    }

    pub fn send_to_random_client(&self, msg: Message) {
        if let Some(client) = self.get_random_client() {
            Clients::handle_message_error(client.unbounded_send(msg));
        }
    }

    pub fn send_recording_list(&self, send_to: SendTo) {
        // Sorted oldest -> newest by modification time, so the last entry is the latest.
        let mut entries: Vec<(std::time::SystemTime, String)> = fs::read_dir("./recordings")
            .unwrap()
            .filter_map(|entry| {
                let e = entry.ok()?;
                let name = e.file_name().to_str()?.to_string();
                let modified = e.metadata().ok()?.modified().ok()?;
                Some((modified, name))
            })
            .collect();
        entries.sort();
        let files: Vec<OscType> = entries
            .into_iter()
            .map(|(_, name)| OscType::String(name))
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
        for (_, tx) in self.client_map.iter() {
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
        let tx = self.client_map.get(&client);
        tx.cloned()
    }

    fn get_random_client(&self) -> Option<Tx> {
        if self.client_array.is_empty() {
            return None;
        }
        let i: usize = rand::random::<usize>() % self.client_array.len();
        let client = self.client_array[i];
        let tx = self.client_map.get(&client);
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
