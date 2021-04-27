mod communication;

use communication::{connection::Connection, general};

fn main() {
    let con = Connection {
        ip: "127.0.0.1".to_string(),
        port: "9090".to_string(),
    };
    match general::ping(&con) {
        Ok(_) => {
            println!("exito")
        }
        Err(e) => {
            println!("no {:?}", e)
        }
    };
}
