mod communication;
use std::env;

use communication::{connection::Connection, general};

fn main() {
    let args: Vec<String> = env::args().collect();

    let dir = format!("archivos-cliente");

    std::fs::create_dir_all(&dir).unwrap();

    let filename: String = args[1].to_owned();

    let con = Connection {
        ip: "127.0.0.1".to_string(),
        port: "9090".to_string(),
    };

    general::descargar_archivo(con, filename, dir);

    // match general::ping(&con) {
    //     Ok(val) => {
    //         println!("exito {}", val)
    //     }
    //     Err(e) => {
    //         println!("no {:?}", e)
    //     }
    // };
}
