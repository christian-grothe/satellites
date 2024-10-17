use futures::channel::mpsc::UnboundedSender;
use std::{
    collections::HashMap,
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

impl Clients {
    pub fn add_client(&mut self, addr: SocketAddr, tx: Tx) {
        self.client_map.lock().unwrap().insert(addr, tx.clone());
        self.client_array.push(addr);
    }

    pub fn remove_client(&mut self, addr: SocketAddr) {
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

    fn get_random_client(&mut self) -> Option<Tx> {
        if self.client_array.is_empty() {
            return None;
        }
        let i: usize = rand::random::<usize>() % self.client_array.len();
        let client = self.client_array[i];
        let binding = self.client_map.lock().unwrap();
        let tx = binding.get(&client);
        tx.cloned()
    }

    pub fn send_to_next_client(&mut self, msg: Message) {
        if let Some(client) = self.get_next_client() {
            if let Err(e) = client.unbounded_send(msg) {
                println!("Error sending message to client: {:?}", e);
            }
        }
    }

    pub fn send_to_random_client(&mut self, msg: Message) {
        if let Some(client) = self.get_random_client() {
            if let Err(e) = client.unbounded_send(msg) {
                println!("Error sending message to client: {:?}", e);
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
}
