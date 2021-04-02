mod communication;

use std::error::Error;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let conexion: TcpStream = TcpStream::connect("127.0.0.1:7777").await?;
    let ip = "127.0.0.1";
    let port = "7777";
    let mut conexion = TcpStream::connect(format!("{}:{}", ip, port).as_str())
        .await
        .unwrap();

    let mut buf = [0; 1024];
    loop {
        // process_socket(socket, addr).await;
        communication::read_stream(&mut conexion, &mut buf).await;
        println!("{:?}", buf);
        communication::receive(communication::write_stream(b"kiubo", &mut conexion).await);
    }

    // Ok(())
}
