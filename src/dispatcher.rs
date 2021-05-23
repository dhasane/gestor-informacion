use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::env;
extern crate serde;
use communication::{connection, distributedfiles, filelist};
use connection::Connection;
use distributedfiles::DistributedFiles;
use filelist::FileList;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

pub mod communication;

lazy_static! {
    static ref REGISTRO: Arc<Mutex<FileList>> = Arc::new(Mutex::new(FileList::create()));
}

pub static PORCENTAJE_DISTRIBUCION: u16 = 20;

// recortar el llamado y evitar que el lock se prolonge
fn get_files() -> Vec<DistributedFiles> {
    REGISTRO.lock().unwrap().clone()
}

/// muestra el registro de archivos que se tiene en el broker
#[get("/getdirs/{filename}")]
async fn get_dirs_filename(web::Path(file_name): web::Path<String>) -> impl Responder {
    let dirs = REGISTRO
        .lock()
        .unwrap()
        .get_connections_by_filename(&file_name);

    println!("{:#?}", dirs);

    let json = serde_json::to_string(&dirs);

    println!("{:?}", json);

    HttpResponse::Ok().body(
        match json {
            Ok(it) => it,
            Err(e) => e.to_string(),
        }
    )
}

/// muestra el registro de archivos que se tiene en el broker
#[get("/get_files")]
async fn get_all_files() -> impl Responder {
    let json = serde_json::to_string(&get_files());
    HttpResponse::Ok().body(
        match json {
            Ok(it) => it,
            Err(_) => "".to_string(),
        }
    )
}

// #[get("/{id}/{name}/index.html")]
// async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
//     format!("Hello {}! id:{}", name, id)
// }

/// Esto es para definir cuando una nueva conexion se genere, de forma
/// que se pueda guardar la direccion.
/// Se recibe el puerto por donde se realizara la respuesta.
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

        let connection = Connection {
            ip: a[..a.find(':').unwrap()].to_owned(),
            port,
        };

        let archivos: Vec<String> = json.0;

        REGISTRO
            .lock()
            .unwrap()
            .agregar_conexion(connection, archivos);
    } else {
        println!("conexion vacia");
    };
    format!("conexion: hola {}", extra)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let port = if args.len() < 1 {
        &args[1]
    } else {
        "8080"
    };

    let ip = format!("0.0.0.0:{port}", port = port);

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(connect)
            .service(get_all_files)
            .service(get_dirs_filename)
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
