mod communication;
use std::path::PathBuf;

use actix_files as fs;
use actix_web::{get, App, Error, HttpRequest, HttpServer, Responder};
use actix_web::{
    http::header::{ContentDisposition, DispositionType},
    web,
};

const BROKER_DIR: &str = "127.0.0.1:8080";

fn serv(dir: &str) -> String {
    format!("http://{}/{}", BROKER_DIR, dir)
}

async fn conectar(port: &str) {
    let url = communication::parse_url(&serv(&format!("connect/{}", port))).unwrap();
    // let respuesta = communication::get(url).await;
    let respuesta = communication::post(url, &files_as_json()).await;
    if let Ok(a) = respuesta {
        println!("{:?}", a);
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all(communication::get_dir()).unwrap();

    let ip = "127.0.0.1";
    let port = "8070";

    conectar(port).await;

    let direccion = format!("{ip}:{port}", ip = ip, port = port);

    println!("iniciando");

    HttpServer::new(|| {
        App::new()
            .service(list_files)
            .service(connect)
            .service(file_serve)
    })
    .bind(direccion)?
    .run()
    .await
}

fn files_as_json() -> String {
    let vec: Vec<String> = communication::get_files();
    let json = serde_json::to_string(&vec);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}

#[get("/list_files")]
async fn list_files() -> impl Responder {
    files_as_json()
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

#[get("file/{file_name}")]
async fn file_serve(web::Path(file_name): web::Path<String>) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf = PathBuf::from(format!(
        "{dir}/{file}",
        dir = communication::get_dir(),
        file = file_name
    ));
    println!("{:?}", path);
    let file = fs::NamedFile::open(path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}
