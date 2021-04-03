mod communication;
use actix_web::{get, App, HttpRequest, HttpServer, Responder};

const BROKER_DIR: &str = "127.0.0.1:8080";

fn serv(dir: &str) -> String {
    format!("http://{}/{}", BROKER_DIR, dir)
}

async fn conectar(port: &str) {
    let url = communication::parse_url(&serv(&format!("connect/{}", port))).unwrap();
    let respuesta = communication::get(url).await;
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

    HttpServer::new(|| App::new().service(list_files).service(connect))
        .bind(direccion)?
        .run()
        .await
}

#[get("/list_files")]
async fn list_files() -> impl Responder {
    let vec: Vec<communication::PathName> = communication::get_files();
    let json = serde_json::to_string(&vec);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
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
