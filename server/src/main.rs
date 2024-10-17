mod clients;
mod osc_server;
mod ws_server;

use clients::Clients;
use osc_server::OscServer;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, UdpSocket};
use ws_server::handle_websocket_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clients_st = Arc::new(Mutex::new(Clients::default()));

    let socket_udp = UdpSocket::bind("0.0.0.0:8081").await?;
    let socket_tcp = TcpListener::bind("0.0.0.0:8080").await?;

    println!("tcp socket listening on {:?}", socket_tcp.local_addr());
    println!("udp socket listening on {:?}", socket_udp.local_addr());

    let osc_server = OscServer {
        socket: socket_udp,
        buf: vec![0; 1024],
        msg: None,
        clients: clients_st.clone(),
    };

    let udp_handler = tokio::spawn(async move {
        let _ = osc_server.run().await;
    });

    let ws_handler = tokio::spawn(async move {
        while let Ok((stream, addr)) = socket_tcp.accept().await {
            tokio::spawn(handle_websocket_connection(
                clients_st.clone(),
                stream,
                addr,
            ));
        }
    });

    udp_handler.await.unwrap();
    ws_handler.await.unwrap();

    Ok(())
}
