mod communication;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./tmp").unwrap();

    let ip = "127.0.0.1:8070";

    HttpServer::new(|| App::new().service(list_files))
        .bind(ip)?
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
