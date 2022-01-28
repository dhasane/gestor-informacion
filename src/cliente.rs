mod communication;
use std::env;

use clap::{App, Arg};
use communication::connection::Connection;

const DISPATCHER_IP: &str = "dispatcher ip";
const DISPATCHER_PORT: &str = "dispatcher port";

fn main() {
    let cli_matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name(DISPATCHER_IP)
                .help("dispatcher ip")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name(DISPATCHER_PORT)
                .help("dispatcher port")
                .long("port")
                .short("p")
                .default_value("8080"),
        )
        .arg(
            Arg::with_name("filename")
                .help("filename")
                .long("filename")
                .short("f")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("send")
                .help("send")
                .long("send")
                .short("s")
                .takes_value(false),
        )
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .get_matches();

    let con = Connection {
        ip: cli_matches.value_of(DISPATCHER_IP).unwrap().to_owned(),
        port: cli_matches.value_of(DISPATCHER_PORT).unwrap().to_owned(),
    };

    let filename: String = cli_matches.value_of("filename").unwrap().to_owned();

    if cli_matches.is_present("send") {
        send(&con, filename);
    } else {
        get(&con, filename);
    }
}

fn send(con: &Connection, filename: String) {
    println!("{}", con.put_file(&filename, 4).unwrap());
}

fn get(con: &Connection, filename: String) {
    let dir = "archivos-cliente".to_string();

    std::fs::create_dir_all(&dir).unwrap();

    println!("{}", con.get_file(&filename, &dir).unwrap());
}
