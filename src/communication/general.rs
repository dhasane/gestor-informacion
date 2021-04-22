#![allow(dead_code)]

use std::fs;

use actix_web::Error;

use reqwest::blocking::Response;
use std::fs::OpenOptions;
use std::io::copy;
use url::Url;

use crate::communication::connection::Connection;

fn get_file_path(filename: &str, ubicacion: &str) -> String {
    format!("{}/{}", ubicacion, sanitize_filename::sanitize(filename))
}

// async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
//     // iterate over multipart stream
//     while let Ok(Some(mut field)) = payload.try_next().await {
//         let content_type = field.content_disposition().unwrap();
//         let filename = content_type.get_filename().unwrap();
//         let filepath = get_file_path(filename);
//
//         // File::create is blocking operation, use threadpool
//         let mut f = web::block(|| std::fs::File::create(filepath))
//             .await
//             .unwrap();
//
//         // Field in turn is stream of *Bytes* object
//         while let Some(chunk) = field.next().await {
//             let data = chunk.unwrap();
//             // filesystem operations are blocking, we have to use threadpool
//             f = web::block(move || f.write_all(&data).map(|_| f)).await?;
//         }
//     }
//     Ok(HttpResponse::Ok().into())
// }

pub fn get_files(ubicacion: String) -> Vec<String> {
    let paths: Vec<String> = fs::read_dir(ubicacion)
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

pub async fn delete_file(file_name: &str, ubicacion: &str) -> Result<(), Error> {
    let filepath = get_file_path(file_name, ubicacion);
    Ok(fs::remove_file(filepath)?)
}

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

pub async fn descargar_archivo(ip_broker: Connection, nombre_archivo: String, ubicacion: String) {
    let ips: Vec<Connection> = pedir_ips_viables(ip_broker, &nombre_archivo)
        .unwrap()
        .iter()
        .map(|f| -> Connection { f.clone() })
        .collect();

    let ip = get_conexion_mas_cercana(ips);

    match download(ip, nombre_archivo, ubicacion) {
        Ok(_) => {
            println!("funciona correctamente")
        }
        Err(e) => {
            println!("Error: {}", e)
        }
    };
}

pub fn download(ip: Connection, nombre_archivo: String, ubicacion: String) -> Result<(), String> {
    println!(
        "Descargando archivo {archivo} de {ip}",
        archivo = nombre_archivo,
        ip = ubicacion
    );

    let url = ip.to_string(format!("file/{}", nombre_archivo));

    // let target = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    // TODO: quitar el blocking cuando sea posible pasar actix-web a usar tokio 1
    let response = match reqwest::blocking::get(url) {
        Ok(it) => it,
        Err(e) => return Err(format!("Error: {:?}", e)),
    };

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        let filepath = format!("{dir}/{file}", dir = ubicacion, file = fname);
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filepath)
        // match File::open(filepath)
        {
            Ok(a) => a,
            Err(e) => return Err(format!("Error: {:?}", e)),
        }
    };
    let content = response.text().unwrap();
    println!("contenido: {}", content);
    match copy(&mut content.as_bytes(), &mut dest) {
        Ok(a) => {
            println!("Resultado exitoso: {}", a);
            Ok(())
        }
        Err(e) => return Err(format!("Error: {:?}", e)),
    }
}
