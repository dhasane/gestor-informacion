use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
extern crate serde;
use communication::{distributedfiles, filelist, general};
use distributedfiles::DistributedFiles;
use filelist::FileList;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

pub mod communication;

lazy_static! {
    static ref REGISTRO: Arc<Mutex<FileList>> = Arc::new(Mutex::new(FileList::create()));
}

// recortar el llamado y evitar que el lock se prolonge
fn get_files() -> Vec<DistributedFiles> {
    REGISTRO.lock().unwrap().clone()
}

#[get("/hello_world")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// muestra el registro de archivos que se tiene en el broker
#[get("/getdirs/{filename}")]
async fn get_dirs_filename(web::Path(file_name): web::Path<String>) -> impl Responder {
    let dirs = REGISTRO
        .lock()
        .unwrap()
        .get_connections_by_filename(&file_name);

    let json = serde_json::to_string(&dirs);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}

/// muestra el registro de archivos que se tiene en el broker
#[get("/get_files")]
async fn get_all_files() -> impl Responder {
    let json = serde_json::to_string(&get_files());
    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}

// #[get("/{id}/{name}/index.html")]
// async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
//     format!("Hello {}! id:{}", name, id)
// }

#[post("connect/{port}")]
async fn connect(
    req: HttpRequest,
    web::Path(port): web::Path<String>,
    json: web::Json<Vec<String>>,
) -> impl Responder {
    let ci = req.connection_info();
    let mut extra = "".to_string();

    if let Some(a) = ci.remote_addr() {
        extra = format!("{}", a);

        let ip: &str = &a[..a.find(':').unwrap()];
        let dir = general::parse_url(&format!("http://{}:{}/connect", ip, port)).unwrap();
        let respuesta = general::get(dir);

        let archivos: Vec<String> = json.0;

        // TODO: conseguir los archivos
        REGISTRO
            .lock()
            .unwrap()
            .agregar_conexion(ip, &port, archivos);

        println!("{:?}", respuesta);
    } else {
        println!("conexion vacia");
    };
    format!("conexion: hola {}", extra)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ip = "127.0.0.1:8080";

    HttpServer::new(|| {
        App::new().service(index).service(connect)
        // .route("/hey", web::get().to(manual_hello))
    })
    .bind(ip)?
    .run()
    .await
}

#[get("/")]
fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
