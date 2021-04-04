mod communication;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
extern crate serde;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

// static mut DATA: Arc<std::sync::Mutex<Vec<communication::Conexion>>>; // = Arc::new(Mutex::new(vec![]));
lazy_static! {
    static ref REGISTRO: Arc<Mutex<FileList>> = Arc::new(Mutex::new(FileList { archivos: vec![] }));
}

pub struct FileList {
    archivos: Vec<communication::Distrib>,
}

impl FileList {
    /// Agrega una conexion y sus archivos al registro.
    /// Puede que haya una manera mas facil, pero de momento esto parece
    /// servir.
    fn agregar_archivo(&mut self, files: Vec<String>, ip: &str, port: &str) {
        let con = communication::Distrib {
            conexion: communication::Connection {
                ip: ip.to_string(),
                port: port.to_string(),
            },
            archivos: files,
        };
        self.archivos.push(con);
    }

    /// Retorna una copia del registro de archivos.
    pub fn clone(&self) -> Vec<communication::Distrib> {
        self.archivos.to_vec()
    }

    /// Conseguir todos los archivos en una conexion especifica.
    /// En caso de no encontrar la conexion, retorna un vector vacio.
    /// Retorna una copia de la lista de archivos de la conexion.
    pub fn get_filenames_by_connection(&self, ip: &str, port: &str) -> Vec<String> {
        let archivos: &Vec<communication::Distrib> = &self.archivos;
        match archivos.iter().find(|&df| df.comp(ip, port)) {
            Some(f) => f.archivos.clone(),
            None => vec![],
        }
    }

    /// Conseguir todas las conexiones que contienen un archivo especifico.
    /// Retorna una copia de la lista conexiones.
    pub fn get_connections_by_filename(&self, nombre: &str) -> Vec<communication::Connection> {
        let archivos: &Vec<communication::Distrib> = &self.archivos;
        archivos
            .iter()
            .filter(|&df| df.archivos.iter().any(|f| f == nombre))
            .map(|f| -> communication::Connection { f.conexion.clone() })
            .collect()
    }
}

fn get_files() -> Vec<communication::Distrib> {
    REGISTRO.lock().unwrap().clone()
}

#[get("/hello_world")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// muestra el registro de archivos que se tiene en el broker
#[get("/get_files")]
async fn get_all_files() -> impl Responder {
    // &get_archivos()
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

#[post("put_file")]
async fn put_file(req: HttpRequest) -> impl Responder {
    // TODO: falta pensar como manejar esto
    // se podria hacer que se suba el archivo al broker de manera
    // temporal mientras es reenviado al destino
    // El broker no deberia almacenar nada de manera permanente
    format!("hola {:?}", req)
}

#[get("get_file/{filename}")]
async fn get_file(web::Path(_file_name): web::Path<String>) -> impl Responder {
    // TODO: falta pensar como manejar esto
    // se podria hacer que se suba el archivo al broker de manera
    // temporal mientras es reenviado al destino
    // El broker no deberia almacenar nada de manera permanente
    "hola"
}

#[get("connect/{port}")]
async fn connect(req: HttpRequest, web::Path(port): web::Path<String>) -> impl Responder {
    let ci = req.connection_info();
    let mut extra = "".to_string();

    if let Some(a) = ci.remote_addr() {
        // TODO: guardar esta conexion
        extra = format!("{}", a);

        let ip: &str = &a[..a.find(':').unwrap()];
        let dir = communication::parse_url(&format!("http://{}:{}/connect", ip, port)).unwrap();
        let respuesta = communication::get(dir).await;

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
        App::new()
            .service(index)
            .service(connect)
            .service(put_file)
            .service(get_file)
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
