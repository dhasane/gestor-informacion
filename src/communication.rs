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

/// Representa una conexion, contiene ip y puerto.
#[derive(Deserialize, Serialize, Clone)]
pub struct Connection {
    pub ip: String,
    pub port: String,
}

impl Connection {
    pub fn to_string(&self, cad: String) -> String {
        format!("http://{}:{}/{}", self.ip, self.port, cad)
    }
}

/// Contiene la conexion y el conjunto de archivos que se encuentran en esta.
#[derive(Deserialize, Serialize, Clone)]
pub struct DistributedFiles {
    pub conexion: Connection,
    pub archivos: Vec<String>,
}

// impl Clone for Distrib {
//     fn clone(&self) -> Self {
//         Distrib{self}
//     }
// }

impl PartialEq for DistributedFiles {
    fn eq(&self, other: &Self) -> bool {
        self.conexion.ip == other.conexion.ip && self.conexion.port == other.conexion.port
    }
}

impl DistributedFiles {
    pub fn comp(&self, ip: &str, port: &str) -> bool {
        self.conexion.ip == ip && self.conexion.port == port
    }

    pub fn contains_file(&self, filename: String) -> bool {
        self.archivos.contains(&filename)
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

pub fn get_files() -> Vec<String> {
    let paths: Vec<String> = fs::read_dir(get_dir())
        .unwrap()
        .map(|r| -> String {
            if let Ok(a) = r {
                format!("{}", a.file_name().into_string().unwrap())
            } else {
                "".to_string()
            }
        })
        .collect();

    paths
}

pub fn parse_json_file_list(json: String) -> Result<Vec<String>, Error> {
    let array: Vec<String> = serde_json::from_str(&json)?;
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
pub fn get(url: Url) -> Result<Response, ()> {
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

pub async fn post(url: Url, json: &str) -> Result<Response, ()> {
    // This will POST a body of `foo=bar&baz=quux`
    // let params = [("foo", "bar"), ("baz", "quux")];
    let client = reqwest::blocking::Client::new();
    match client.post(url.as_str()).form(json).send() {
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

/// conseguir lista de direcciones viables que contienen un
/// archivo especifico desde broker
pub fn pedir_ips_viables(
    ip_broker: Connection,
    nombre_archivo: &str,
) -> Result<Vec<Connection>, reqwest::Error> {
    let url = ip_broker.to_string(format!("getdirs/{}", nombre_archivo));

    // TODO: llenar vect con las posibles ips
    let respuesta: reqwest::blocking::Response = reqwest::blocking::get(url)?;
    println!("{:?}", respuesta);

    let json: String = match respuesta.json() {
        Ok(a) => a,
        _ => "".to_string(),
    };
    let ret: Vec<Connection> = serde_json::from_str(&json).unwrap();
    Ok(ret)
}

fn get_conexion_mas_cercana(conexiones_posibles: Vec<Connection>) -> Connection {
    let ret: Connection;

    // TODO: hacer ping a cada una de las ips y medir el tiempo de cada una
    // retornar la ip que responda mas rapido

    // eliminar las ips de forma automatica que superen un tiempo dado
    let con = conexiones_posibles.get(0);

    ret = con.unwrap().to_owned();
    ret
}

pub fn descargar_archivo(ip_broker: Connection, nombre_archivo: &str) {
    let ubicacion = "";

    let ips: Vec<Connection> = pedir_ips_viables(ip_broker, nombre_archivo)
        .unwrap()
        .iter()
        .map(|f| -> Connection { f.clone() })
        .collect();

    let ip = get_conexion_mas_cercana(ips);

    download(ip, nombre_archivo, ubicacion);
}

pub fn download(ip: Connection, nombre_archivo: &str, ubicacion: &str) {
    // TODO: pedir nombre_archivo a ip
    // finalmente se guarda en ubicacion
}
