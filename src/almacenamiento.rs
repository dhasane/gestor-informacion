use std::{env, path::PathBuf, time::SystemTime};

use actix_files as fs;
use actix_web::{get, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::{
    http::header::{ContentDisposition, DispositionType},
    web,
};
use communication::{connection, general};
pub mod communication;
use connection::Connection;

const BROKER_DIR: &str = "127.0.0.1:8080";

static mut PRUEBA: Option<String> = None;

pub fn set_dir(dir: String) {
    unsafe {
        PRUEBA = Some(dir);
    }
}

pub fn get_dir() -> String {
    unsafe {
        if let Some(dir) = &PRUEBA {
            format!("./{dir}", dir = dir)
        } else {
            "".to_string()
        }
    }
}

fn serv(dir: &str) -> String {
    format!("http://{}/{}", BROKER_DIR, dir)
}

async fn conectar(port: &str) {
    let url = general::parse_url(&serv(&format!("connect/{}", port))).unwrap();
    // let respuesta = general::get(url).await;
    let respuesta = general::post(url, &files_as_json(get_dir())).await;
    if let Ok(a) = respuesta {
        println!("{:?}", a);
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 1 {
        println!("Error: es necesario especificar el puerto");
        return Ok(());
    }

    let ip = "0.0.0.0";
    let port = &args[1];

    set_dir(format!("tmp-{}", port));

    std::fs::create_dir_all(get_dir()).unwrap();

    conectar(&port).await;

    let direccion = format!("{ip}:{port}", ip = ip, port = port);

    println!("iniciando");

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

fn files_as_json(ubicacion: String) -> String {
    let vec: Vec<String> = general::get_files(ubicacion);
    let json = serde_json::to_string(&vec);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}

#[get("/list_files")]
async fn list_files() -> impl Responder {
    files_as_json(get_dir())
}

#[get("/ping")]
async fn ping_listener() -> impl Responder {
    format!("{:?}", SystemTime::now())
}

// esto es una prueba
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

/// Pide al almacenamiento que consiga el archivo file_name encontrado en ip:port
#[get("go_get_file/{ip}:{port}/{file_name}")]
async fn go_get_file(
    web::Path((ip, port, file_name)): web::Path<(String, String, String)>,
) -> impl Responder {
    let url = Connection { ip, port };
    match general::download(url, file_name, get_dir()) {
        Ok(_) => {
            format!("Archivo descargado")
        }
        Err(e) => {
            format!("{}", e)
        }
    }
}

#[get("file/{file_name}")]
async fn file_serve(web::Path(file_name): web::Path<String>) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf =
        PathBuf::from(format!("{dir}/{file}", dir = get_dir(), file = file_name));
    println!("{:?}", path);
    let file = fs::NamedFile::open(path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

#[get("/")]
fn index() -> HttpResponse {
    let start = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <h1> Archivos: </h1>
            <ul>
        "#;

    let vec: Vec<String> = general::get_files(get_dir());

    let mid: String = vec.iter().map(|f| format!("<li>{}</li>", f)).collect();

    let end = r#"
            </ul>
        </body>
        </html>"#;

    let html = format!("{}{}{}", start, mid, end);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
