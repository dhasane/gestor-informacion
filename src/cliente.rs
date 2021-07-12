mod communication;
use std::env;

use communication::connection::Connection;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Se debe especificar [ip dispatcher] [puerto dispatcher] [archivo]");
        return;
    }

    let dir = "archivos-cliente".to_string();

    std::fs::create_dir_all(&dir).unwrap();

    let con = Connection {
        ip: args[1].to_owned(),
        port: args[2].to_owned(),
    };

    let filename: String = args[3].to_owned();

    println!("{}", con.get_file(filename, dir).unwrap());
}
