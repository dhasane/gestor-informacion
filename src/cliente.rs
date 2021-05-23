mod communication;
use std::env;

use communication::{connection::Connection, general};

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Se debe especificar [ip broker] [puerto broker] [archivo]");
        return ;
    }

    let dir = format!("archivos-cliente");

    std::fs::create_dir_all(&dir).unwrap();

    let con = Connection {
        ip: args[1].to_owned(),
        port: args[2].to_owned(),
    };

    let filename: String = args[3].to_owned();

    general::get_file(&con, filename, dir);

}
