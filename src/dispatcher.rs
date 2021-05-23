use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, rt::{spawn, time}, web};
use std::{env, time::Duration};
extern crate serde;
use communication::{connection, distributedfiles, filelist};
use connection::Connection;
use distributedfiles::DistributedFiles;
use filelist::FileList;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use rand::Rng;

pub mod communication;

lazy_static! {
    static ref REGISTRO: Arc<Mutex<FileList>> = Arc::new(Mutex::new(FileList::create()));
}

// tiempo en segundos para balancear
pub static TIEMPO_BALANCEO: u64 = 30;

pub static PORCENTAJE_DISTRIBUCION: u16 = 20;

pub static MINIMO_NUMERO_ARCHIVOS: u64 = 3;

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

    let json = serde_json::to_string(&dirs);
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
            .agregar_o_reemplazar_conexion(connection, archivos);
    } else {
        println!("conexion vacia");
    };
    format!("conexion: hola {}", extra)
}

// TODO: esto podria valer la pena quitarlo
// o aunuque sea hacerlo util
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

fn go_get(con: Connection, nombre_archivo: &str) {
    let url = con.to_string(format!("go_get_file/{}", nombre_archivo));
    let respuesta = match reqwest::blocking::get(url) {
        Ok(it) => it.text().unwrap(),
        Err(e) => {
            format!("Error de conexion:\n{:?}", e)
        }
    };
    println!("{}", respuesta);
}

fn balancear() {
    let numer_archivos: Vec<(String, u64)> = REGISTRO
        .lock()
        .unwrap()
        .get_number_of_files();
    println!("{:?}", numer_archivos);

    let mut rng = rand::thread_rng();

    for (nombre, cantidad ) in numer_archivos {

        println!("{} {}", cantidad, MINIMO_NUMERO_ARCHIVOS);
        let mut diferencia = if cantidad >= MINIMO_NUMERO_ARCHIVOS {
            0
        } else {
            MINIMO_NUMERO_ARCHIVOS - cantidad
        };

        if diferencia != 0 {

            let mut conexiones_viables: Vec<Connection> = REGISTRO
                .lock()
                .unwrap()
                .get_connections_without_filename(&nombre);

            println!("{:?}", conexiones_viables);

            while diferencia > 0 && conexiones_viables.len() > 0 {
                let pos = rng.gen_range(0..conexiones_viables.len());

                let conexion: Connection = conexiones_viables.remove(pos);

                go_get(conexion, &nombre);

                diferencia -= 1;

            }
        }

    }
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

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(TIEMPO_BALANCEO));
        loop {
            interval.tick().await;
            balancear();
        }
    });

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
