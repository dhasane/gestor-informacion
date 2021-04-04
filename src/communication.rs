#![allow(dead_code)]

use std::fs;
use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use futures::{StreamExt, TryStreamExt};

use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};
use url::Url;

const DIRNAME: &str = "tmp";

pub fn get_dir() -> String {
    format!("./{dir}", dir = DIRNAME)
}

#[derive(Deserialize, Serialize)]
pub struct PathName {
    pub nombre: String,
}

// se guarda una linea por la relacion entre archivo y conexion
// a pesar de crear una lista mas larga, facilita la busqueda con base
// a nombre o conexion
pub struct DistributedFile {
    pub nombre: String,
    pub conexion: Distrib,
}

/// Representa una conexion, contiene ip y puerto.
#[derive(Deserialize, Serialize, Clone)]
pub struct Connection {
    pub ip: String,
    pub port: String,
}

/// Contiene la conexion y el conjunto de archivos que se encuentran en esta.
#[derive(Deserialize, Serialize, Clone)]
pub struct Distrib {
    pub conexion: Connection,
    pub archivos: Vec<String>,
}

// impl Clone for Distrib {
//     fn clone(&self) -> Self {
//         Distrib{self}
//     }
// }

impl PartialEq for Distrib {
    fn eq(&self, other: &Self) -> bool {
        self.conexion.ip == other.conexion.ip && self.conexion.port == other.conexion.port
    }
}

impl Distrib {
    pub fn comp(&self, ip: &str, port: &str) -> bool {
        self.conexion.ip == ip && self.conexion.port == port
    }
}

fn get_file_path(filename: &str) -> String {
    format!("{}/{}", get_dir(), sanitize_filename::sanitize(filename))
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = get_file_path(filename);

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

pub fn get_files() -> Vec<PathName> {
    let paths: Vec<PathName> = fs::read_dir(get_dir())
        .unwrap()
        .map(|r| -> PathName {
            PathName {
                nombre: if let Ok(a) = r {
                    format!("{}", a.path().display())
                } else {
                    "".to_string()
                },
            }
        })
        .collect();

    paths
}

pub fn parse_json_file_list(json: String) -> Result<Vec<PathName>, Error> {
    let array: Vec<PathName> = serde_json::from_str(&json)?;
    Ok(array)
}

pub async fn delete_file(file_name: &str) -> Result<(), Error> {
    let filepath = get_file_path(file_name);
    Ok(fs::remove_file(filepath)?)
}
//
// pub async fn getFile(id: u32) -> File {}

pub fn parse_url(url: &str) -> Result<Url, ()> {
    match Url::parse(url) {
        Ok(a) => Ok(a),
        Err(_) => Err(()),
    }

    // println!("error parse url: {}", err);
    // Url {
    //     // serialization: (),
    //     // scheme_end: (),
    //     // username_end: (),
    //     // host_start: (),
    //     // host_end: (),
    //     // host: (),
    //     port: Some(port),
    //     path_start: path,
    //     // query_start: (),
    //     // fragment_start: (),
    // }
}

/// Realizar operacion de GET y retornar el resultado.
/// Realmente solo es para recordar.
pub async fn get(url: Url) -> Result<Response, ()> {
    println!("{}", url);
    let response;
    match reqwest::blocking::get(url.as_str()) {
        Ok(a) => {
            response = a;
        }
        Err(err) => {
            println!("error: {}", err);
            return Err(());
        }
    };

    Ok(response)
}

pub async fn post(url: Url) -> Result<Response, ()> {
    // This will POST a body of `foo=bar&baz=quux`
    let params = [("foo", "bar"), ("baz", "quux")];
    let client = reqwest::blocking::Client::new();
    match client.post(url.as_str()).form(&params).send() {
        Ok(a) => Ok(a),
        Err(err) => {
            println!("post error: {}", err);
            Err(())
        }
    }
}
//
// pub async fn post() -> Result<String> {
//     let body = reqwest::get("https://www.rust-lang.org")
//         .await?
//         .text()
//         .await?;
//
//     println!("body = {:?}", body);
//     Ok(body)
// }
