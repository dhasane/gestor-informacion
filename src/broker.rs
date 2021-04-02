mod communication;

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7777").await.unwrap();
    // let mut conexiones: Vec<(&TcpStream, &SocketAddr)>;

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        // conexiones.push((&socket, &addr));
        tokio::spawn(async move {
            // process_socket(socket, addr).await;
            saludo(&mut socket, &addr).await;
            listen_socket(&mut socket).await;
        });
    }
}

async fn listen_socket(conexion: &mut TcpStream) {
    let mut buf = [0; 1024];
    loop {
        // process_socket(socket, addr).await;
        communication::read_stream(conexion, &mut buf).await;
        println!("{:?}", buf);
    }
}

async fn saludo(socket: &mut TcpStream, addr: &SocketAddr) {
    println!("Nueva conexion: {}", addr);
    communication::receive(communication::write_stream(b"holaa", socket).await)
}
