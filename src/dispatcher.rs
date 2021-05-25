use actix_web::{
    get, post,
    rt::{spawn, time},
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use std::{env, time::Duration};
extern crate serde;
use communication::{connection, distributedfiles, filelist};
use connection::Connection;
use distributedfiles::DistributedFiles;
use filelist::FileList;
use lazy_static::lazy_static;
use rand::Rng;
use std::sync::{Arc, Mutex};

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
    HttpResponse::Ok().body(match json {
        Ok(it) => it,
        Err(e) => e.to_string(),
    })
}

/// muestra el registro de archivos que se tiene en el broker
#[get("/get_files")]
async fn get_all_files() -> impl Responder {
    let json = serde_json::to_string(&get_files());
    HttpResponse::Ok().body(match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    })
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

#[get("/")]
fn index() -> HttpResponse {
    let numero_archivos: String = REGISTRO
        .lock()
        .unwrap()
        .get_number_of_files()
        .iter()
        .map(|(nombre, cantidad)| -> String {
            format!("<tr><th>{}</th><th>{}</th></tr>", nombre, cantidad)
        })
        .collect::<String>();

    let conexion_archivos: String = REGISTRO
        .lock()
        .unwrap()
        .clone()
        .iter()
        .map(|distrib_file| -> String {
            format!(
                "conexion: {conexion} <ul> {archivos} </ul>",
                conexion = distrib_file.conexion,
                archivos = distrib_file
                    .archivos
                    .iter()
                    .map(|a| -> String { format!("<li>{}</li>", a) })
                    .collect::<String>()
            )
        })
        .collect();

    let html = format!(
        r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <h1> Archivos y cantidad </h1>
                <table>
                <tr>
                    <th> archivo </th>
                    <th> cantidad </th>
                </tr>
                    {numero_archivos}
                </table>
            <h1> Conexion y archivos </h1>
            {con_arch}
        </body>
    </html>"#,
        numero_archivos = numero_archivos,
        con_arch = conexion_archivos
    );

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
    let numero_archivos: Vec<(String, u64)> = REGISTRO.lock().unwrap().get_number_of_files();
    println!("{:?}", numero_archivos);

    let cantidad_conexiones = REGISTRO.lock().unwrap().size();

    let mut rng = rand::thread_rng();

    for (nombre, cantidad) in numero_archivos {
        let mut diferencia =
            if cantidad >= MINIMO_NUMERO_ARCHIVOS || cantidad as usize >= cantidad_conexiones {
                0
            } else {
                MINIMO_NUMERO_ARCHIVOS - cantidad
            };

        if diferencia != 0 {
            let mut conexiones_viables: Vec<Connection> = REGISTRO
                .lock()
                .unwrap()
                .get_connections_without_filename(&nombre);

            println!(
                "conexiones para enviar archivo {} {:?}",
                nombre, conexiones_viables
            );

            while diferencia > 0 && conexiones_viables.len() > 0 {
                let pos = rng.gen_range(0..conexiones_viables.len());

                let conexion: Connection = conexiones_viables.remove(pos);

                println!("{} <- {}", conexion, nombre);
                go_get(conexion, &nombre);

                diferencia -= 1;
            }
            println!("================================");
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let port = if args.len() < 1 { &args[1] } else { "8080" };

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
    })
    .bind(ip)?
    .run()
    .await
}
