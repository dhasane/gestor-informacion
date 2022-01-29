use actix_web::{
    get, post,
    rt::{spawn, time},
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use std::{env, time::Duration};
extern crate serde;
use communication::{connection, filelist};
use connection::Connection;
use filelist::FileList;
use lazy_static::lazy_static;
use rand::{seq::SliceRandom, Rng};
use std::sync::{Arc, Mutex};

use crate::communication::network;

pub mod communication;

const FILELIST_FILE: &str = "./filelist.json";

lazy_static! {
    static ref REGISTRO: Arc<Mutex<FileList>> = Arc::new(Mutex::new(FileList::load(FILELIST_FILE)));
}

// tiempo en segundos para balancear
pub static TIEMPO_BALANCEO: u64 = 30;

// porcentaje del total despues de superar el minimo, hasta llegar a un maximo
pub static PORCENTAJE_DISTRIBUCION: f64 = 0.5;

pub static MINIMO_NUMERO_ARCHIVOS: u64 = 3;
pub static MAXIMO_NUMERO_ARCHIVOS: u64 = 20;

/// Retorna todas las conexiones en el registro que contengan un archivo especifico.
#[get("/get_connections/{filename}")]
async fn get_connections_filename(web::Path(file_name): web::Path<String>) -> impl Responder {
    let dirs: Vec<Connection> = REGISTRO
        .lock()
        .unwrap()
        .get_connections_by_filename(&file_name);

    let json = serde_json::to_string(&dirs);
    HttpResponse::Ok().body(match json {
        Ok(it) => it,
        Err(e) => e.to_string(),
    })
}

#[get("/get_random_connections/{number}")]
async fn get_connections(web::Path(number): web::Path<usize>) -> impl Responder {
    let dirs: Vec<Connection> = REGISTRO.lock().unwrap().get_connections();

    let rnd_values: Vec<Connection> = dirs
        .choose_multiple(&mut rand::thread_rng(), number)
        .cloned()
        .collect();

    let json = serde_json::to_string(&rnd_values);
    HttpResponse::Ok().body(match json {
        Ok(it) => it,
        Err(e) => e.to_string(),
    })
}

/// muestra el registro de archivos que se tiene en el dispatcher
#[get("/get_all_files")]
async fn get_all_files() -> impl Responder {
    let json = serde_json::to_string(REGISTRO.lock().unwrap().get_files());
    HttpResponse::Ok().body(match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    })
}

/// Esto es para definir cuando una nueva conexion se genere, de forma
/// que se pueda guardar la direccion.
/// Se recibe el puerto por donde se realizara la respuesta.
#[post("send_files/{port}")]
async fn receive_files(
    req: HttpRequest,
    web::Path(port): web::Path<String>,
    json: web::Json<Vec<String>>,
) -> impl Responder {
    let con_info = req.connection_info();

    if let Some(remote_addr) = con_info.remote_addr() {
        let connection = Connection {
            ip: remote_addr[..remote_addr.find(':').unwrap()].to_owned(),
            port,
        };

        let archivos: Vec<String> = json.0;

        REGISTRO
            .lock()
            .unwrap()
            .add_or_replace_connection(connection, archivos);
    } else {
        println!("conexion vacia");
    };
    "hola".to_string()
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
        .get_files()
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

/// Pedir a CON, que vaya y consiga el archivo FILENAME
fn go_get(con: &Connection, filename: &str) {
    let url = con.to_url(format!("go_get_file/{}", filename));
    let respuesta = match network::get(&url) {
        Ok(it) => it.text().unwrap(),
        Err(e) => {
            format!("Error de conexion:\n{:?}", e)
        }
    };
    println!("{}", respuesta);
}

/// Realiza el balanceo de archivos, enviando los archivos que no
/// tengan suficientes ocurrencias dentro del sistema
fn balancear() {
    let numero_archivos: Vec<(String, u64)> = REGISTRO.lock().unwrap().get_number_of_files();
    println!("{:?}", numero_archivos);

    let cantidad_conexiones = REGISTRO.lock().unwrap().size();

    let mut rng = rand::thread_rng();

    for (nombre, cantidad) in numero_archivos {
        let porc_min = (cantidad_conexiones as f64 * PORCENTAJE_DISTRIBUCION) as u64;

        // TODO: revisar los porcentajes minimos de distribucion de archivos hasta el maximo
        let mut diferencia =
            if cantidad >= MINIMO_NUMERO_ARCHIVOS || cantidad >= cantidad_conexiones {
                0
            } else if cantidad < MINIMO_NUMERO_ARCHIVOS {
                MINIMO_NUMERO_ARCHIVOS - cantidad
            } else if cantidad < porc_min {
                porc_min - cantidad
            } else {
                0
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

            while diferencia > 0 && !conexiones_viables.is_empty() {
                let pos = rng.gen_range(0..conexiones_viables.len());

                let conexion: Connection = conexiones_viables.remove(pos);

                println!("{} <- {}", conexion, nombre);
                go_get(&conexion, &nombre);

                diferencia -= 1;
            }
            println!("================================");
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let port = if args.len() > 1 { &args[1] } else { "8080" };

    println!("iniciando dispatcher en puerto {}", port);

    let ip = format!("0.0.0.0:{port}", port = port);

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(TIEMPO_BALANCEO));
        loop {
            interval.tick().await;
            // TODO: hacer algo para verificar si los almacenamientos siguen disponibles
            // se podria hacer un ping a cada uno y en caso de no
            // responder o tomar mas tiempo del necesario, se elimina
            // del registro
            balancear();
            println!(
                "{}",
                match REGISTRO.lock().unwrap().store(FILELIST_FILE) {
                    Ok(a) => a,
                    Err(e) => e,
                },
            )
        }
    });

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(receive_files)
            .service(get_all_files)
            .service(get_connections_filename)
            .service(get_connections)
    })
    .bind(ip)?
    .run()
    .await
}
