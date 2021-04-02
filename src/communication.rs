use io::Error;
use std::io::{self, prelude::*};
use std::{fs::File, path::Path};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{unix::SocketAddr, TcpStream},
};

pub fn receive(ret: Result<usize, Error>) {
    match ret {
        Ok(a) => println!("operacion exitosa: {}", a),
        Err(err) => eprintln!("error en operacion: {}", err),
    }
}

// enviar archivos
pub async fn send_file(
    stream: &mut TcpStream,
    addr: &SocketAddr,
    path: &str,
) -> Result<usize, Error> {
    let path = Path::new(path);
    let file_name = path.file_name().unwrap();
    println!("Enviando: {:?} a {:?}", file_name, addr);
    let mut file = File::open(file_name)?;
    // read the same file back into a Vec of bytes
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;
    // self.stream.write(&buffer).await;
    write_stream(&buffer, stream).await
    // Ok(())
}

pub async fn write_stream(buffer: &[u8], stream: &mut TcpStream) -> Result<usize, Error> {
    stream.write(buffer).await
}

pub async fn read_stream(stream: &mut TcpStream, mut buf: &mut [u8]) {
    let n = match stream.read(&mut buf).await {
        // socket closed
        Ok(n) if n == 0 => return,
        Ok(n) => n,
        Err(e) => {
            eprintln!("failed to read from socket; err = {:?}", e);
            return;
        }
    };

    println!("{} -> {:?}", n, buf);

    // Write the data back
    // if let Err(e) = conexion.write_all(&buf[0..n]).await {
    //     eprintln!("failed to write to socket; err = {:?}", e);
    //     return;
    // }
}
