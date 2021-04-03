mod communication;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
extern crate serde;

#[get("/hello_world")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

#[get("list_files")]
async fn get_files() -> impl Responder {
    // TODO: para cada una de las direcciones que se tienen, realizar un
    // GET /list_files, organizarlo en una unica lista y retornarlo

    // https://doc.rust-lang.org/std/sync/struct.Mutex.html

    // pensar en una forma para que esto sea hecho de manera
    // periodica, para que no haya que pedir los archivos solo cuando
    // son necesarios, sino que se tengan desde antes
    "ye"
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
