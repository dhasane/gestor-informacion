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
async fn get_file(web::Path(file_name): web::Path<String>) -> impl Responder {
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
        let dir = format!("http://{}:{}/connect", ip, port);
        let respuesta = communication::get(&dir).await;

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
