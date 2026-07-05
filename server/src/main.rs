mod clients;
mod osc_server;
mod ws_server;

use clients::Clients;
use notify::{Event, RecursiveMode, Watcher};
use osc_server::OscServer;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio_tungstenite::tungstenite::Message;
use ws_server::handle_websocket_connection;

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PONG_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clients = Arc::new(Mutex::new(Clients::default()));
    let clients_watcher = clients.clone();

    let socket_udp = UdpSocket::bind("0.0.0.0:8081").await?;
    let socket_tcp = TcpListener::bind("0.0.0.0:8080").await?;

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

            clients.broadcast(Message::Ping(vec![]));
        }
    };

    let ping_handler = tokio::spawn(ping_task);

    println!("tcp socket listening on {:?}", socket_tcp.local_addr());
    println!("udp socket listening on {:?}", socket_udp.local_addr());

    let osc_server = OscServer {
        socket: socket_udp,
        buf: vec![0; 1024],
        msg: None,
        clients: clients.clone(),
    };

    let mut watcher = notify::recommended_watcher({
        move |res| match res {
            Ok(event) => handle_watch_event(event, clients_watcher.clone()),
            Err(e) => println!("watch error: {:?}", e),
        }
    })?;

    let watch_handler = tokio::spawn(async move {
        if let Err(e) = watcher.watch(Path::new("./recordings"), RecursiveMode::Recursive) {
            println!("watch error: {:?}", e);
            return;
        }
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    let udp_handler = tokio::spawn(async move {
        let _ = osc_server.run().await;
    });

    let ws_handler = tokio::spawn(async move {
        while let Ok((stream, addr)) = socket_tcp.accept().await {
            tokio::spawn(handle_websocket_connection(clients.clone(), stream, addr));
        }
    });

    tokio::try_join!(watch_handler, udp_handler, ws_handler, ping_handler)?;

    Ok(())
}

fn handle_watch_event(event: Event, clients: Arc<Mutex<Clients>>) {
    match event.kind {
        notify::EventKind::Create(_) => {
            let clients = clients.lock().unwrap();
            clients.send_recording_list(clients::SendTo::Broadcast);
        }
        _ => {}
    }
}
