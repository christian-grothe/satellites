mod clients;
mod osc_server;
mod ws_server;

use clients::Clients;
use notify::{Event, RecursiveMode, Watcher};
use osc_server::OscServer;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, UdpSocket};
use tokio_tungstenite::tungstenite::Message;
use ws_server::handle_websocket_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clients = Arc::new(Mutex::new(Clients::default()));
    let clients_watcher = clients.clone();

    let socket_udp = UdpSocket::bind("0.0.0.0:8081").await?;
    let socket_tcp = TcpListener::bind("0.0.0.0:8080").await?;

    println!("tcp socket listening on {:?}", socket_tcp.local_addr());
    println!("udp socket listening on {:?}", socket_udp.local_addr());

    let mut watcher = notify::recommended_watcher({
        move |res| match res {
            Ok(event) => handle_watch_event(event, clients_watcher.clone()),
            Err(e) => println!("watch error: {:?}", e),
        }
    })?;

    let osc_server = OscServer {
        socket: socket_udp,
        buf: vec![0; 1024],
        msg: None,
        clients: clients.clone(),
    };

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

    watch_handler.await.unwrap();
    udp_handler.await.unwrap();
    ws_handler.await.unwrap();

    Ok(())
}

fn handle_watch_event(event: Event, clients: Arc<Mutex<Clients>>) {
    match event.kind {
        notify::EventKind::Create(_) => {
            let clients = clients.lock().unwrap();
            let msg = Message::Text("Hello".to_string());
            clients.broadcast(msg);
        }
        _ => {}
    }
}
