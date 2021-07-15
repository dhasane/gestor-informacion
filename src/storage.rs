use std::{env, path::PathBuf, process, time::SystemTime};

use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionType},
    post, web, App, Error, HttpResponse, HttpServer, Responder,
};
use communication::{connection, general};
pub mod communication;
use connection::Connection;

use std::io::Write;

use futures::{StreamExt, TryStreamExt};

pub struct Config {
    /// Conexion al dispatcher
    pub dispatcher: Connection,
    /// Directorio en el cual se guardaran los archivos
    pub directorio: String,
    /// Puerto desde donde se reciben mensajes
    pub puerto: String,
}

static mut CONFIGURACION: Option<Config> = None;

fn set_config(dispatcher: Connection, directorio: String, puerto: String) {
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

pub fn get_dispatcher_con() -> &'static Connection {
    unsafe {
        if let Some(config) = &CONFIGURACION {
            &config.dispatcher
        } else {
            println!("Error consiguiendo conexion a dispatcher, falta configurar");
            process::exit(1);
        }
    }
}

pub fn get_port() -> &'static str {
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
fn send_file_list() {
    let connection = get_dispatcher_con();
    let port = get_port();
    println!("enviando archivos a {}", connection.base_str());

    let respuesta = general::send_files(connection, format!("send_files/{}", port), get_dir());

    match respuesta {
        Ok(_a) => {
            println!("archivos enviados exitosamente");
        }
        Err(e) => println!("{:?}", e),
    };
}

/// Responde al ""ping""
#[get("/ping")]
async fn ping_listener() -> impl Responder {
    format!("Ping: {:?}", SystemTime::now())
}

/// Pide al almacenamiento que consiga el archivo file_name
#[get("go_get_file/{file_name}")]
async fn go_get_file(web::Path(file_name): web::Path<String>) -> impl Responder {
    // revisar si no se tiene ya el archivo
    if !general::get_files_in_dir(get_dir())
        .iter()
        .any(|f| f == &file_name)
    {
        let result = get_dispatcher_con().get_file(file_name, get_dir());
        send_file_list();
        result.unwrap()
    } else {
        "no se descargo el archivo".to_string()
    }
}

#[get("download/{file_name}")]
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

// https://github.com/actix/examples/blob/a66c05448eace8b1ea53c7495b27604e7e91281c/forms/multipart/src/main.rs
#[post("upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!(
            "./{dir}/{path}",
            dir = get_dir(),
            path = sanitize_filename::sanitize(&filename)
        );

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    Ok(HttpResponse::Ok().into())
}

#[get("/")]
fn index() -> HttpResponse {
    let vec: Vec<String> = general::get_files_in_dir(get_dir());

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

    send_file_list();

    let direccion = format!("{ip}:{port}", ip = ip, port = port);

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(file_serve)
            .service(go_get_file)
            .service(ping_listener)
            .service(upload)
    })
    .bind(direccion)?
    .run()
    .await
}
