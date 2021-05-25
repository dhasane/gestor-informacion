use std::{env, path::PathBuf, process, time::SystemTime};

use actix_files as fs;
use actix_web::{get, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::{
    http::header::{ContentDisposition, DispositionType},
    web,
};
use communication::{connection, general};
pub mod communication;
use connection::Connection;

pub struct Config {
    /// Conexion al dispatcher
    pub dispatcher: Connection,
    /// Directorio en el cual se guardaran los archivos
    pub directorio: String,
    /// Puerto desde donde se reciben mensajes
    pub puerto: String,
}

static mut CONFIGURACION: Option<Config> = None;

pub fn set_config(dispatcher: Connection, directorio: String, puerto: String) {
    unsafe {
        CONFIGURACION = Some(Config {
            dispatcher,
            directorio,
            puerto,
        })
    }
}

pub fn get_dir() -> String {
    unsafe {
        if let Some(config) = &CONFIGURACION {
            format!("./{dir}", dir = config.directorio)
        } else {
            println!("Error consiguiendo directorio, falta configurar");
            process::exit(1);
        }
    }
}

pub fn get_dispatcher_dir() -> &'static Connection {
    unsafe {
        if let Some(config) = &CONFIGURACION {
            &config.dispatcher
        } else {
            println!("Error consiguiendo conexion a dispatcher, falta configurar");
            process::exit(1);
        }
    }
}

pub fn get_puerto() -> &'static str {
    unsafe {
        if let Some(config) = &CONFIGURACION {
            &config.puerto
        } else {
            println!("Error consiguiendo puerto, falta configurar");
            process::exit(1);
        }
    }
}

/// Se envia el puerto, de forma que el dispatcher sepa por donde responder.
/// Se envian los archivos pertenecientes al almacenamiento local.
fn enviar_archivos() {
    let connection = get_dispatcher_dir();
    let port = get_puerto();
    println!("enviando archivos a {}", connection.base_str());

    let respuesta = general::send_files(connection, format!("connect/{}", port), get_dir());

    match respuesta {
        Ok(_a) => {
            println!("archivos enviados exitosamente");
        }
        Err(e) => println!("{:?}", e),
    };
}

#[get("/list_files")]
async fn list_files() -> impl Responder {
    format!("{:?}", general::files_as_json(get_dir()))
}

#[get("/ping")]
async fn ping_listener() -> impl Responder {
    format!("{:?}", SystemTime::now())
}

#[get("connect")]
async fn connect(req: HttpRequest) -> impl Responder {
    let ci = req.connection_info();
    let mut extra = "".to_string();
    if let Some(a) = ci.remote_addr() {
        println!("conexion exitosa: {}", a);
        extra = format!("{}", a);
    } else {
        println!("conexion vacia");
    }
    format!("conexion: hola {}", extra)
}

/// Pide al almacenamiento que consiga el archivo file_name
#[get("go_get_file/{file_name}")]
async fn go_get_file(web::Path(file_name): web::Path<String>) -> impl Responder {
    if !general::get_files(get_dir())
        .iter()
        .any(|f| f == &file_name)
    {
        let ret = general::get_file(get_dispatcher_dir(), file_name, get_dir());
        enviar_archivos();
        ret
    } else {
        "no se descargo el archivo".to_string()
    }
}

#[get("file/{file_name}")]
async fn file_serve(web::Path(file_name): web::Path<String>) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf =
        PathBuf::from(format!("{dir}/{file}", dir = get_dir(), file = file_name));
    println!("{:?}", path);
    let file = fs::NamedFile::open(path)?;

    println!("Se descarga archivo {file}", file = file_name);

    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

#[get("/")]
fn index() -> HttpResponse {
    let vec: Vec<String> = general::get_files(get_dir());

    let archivos: String = vec.iter().map(|f| format!("<li>{}</li>", f)).collect();

    let html = format!(
        "<html>
        <head><title>Upload Test</title></head>
        <body>
            <h1> Archivos: </h1>
            <ul>
                {}
            </ul>
        </body>
        </html> ",
        archivos
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Se debe especificar [puerto] [ip dispatcher] [puerto dispatcher]");
        return Ok(());
    }

    let ip = "0.0.0.0";
    let port = &args[1];

    let dispatcher_ip = &args[2];
    let dispatcher_port = &args[3];

    set_config(
        Connection {
            ip: dispatcher_ip.to_owned(),
            port: dispatcher_port.to_owned(),
        },
        format!("tmp-{}", port),
        port.to_owned(),
    );

    std::fs::create_dir_all(get_dir()).unwrap();

    enviar_archivos();

    let direccion = format!("{ip}:{port}", ip = ip, port = port);

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(list_files)
            .service(connect)
            .service(file_serve)
            .service(go_get_file)
            .service(ping_listener)
    })
    .bind(direccion)?
    .run()
    .await
}
