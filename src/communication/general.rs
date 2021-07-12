#![allow(dead_code)]
use std::fs;

use reqwest::blocking::Response;

use crate::communication::connection::Connection;

/// Conseigue los archivos que se encuentran en UBICACION y retorna una lista de strings
pub fn get_files_in_dir(dir: String) -> Vec<String> {
    let paths: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|r| -> String {
            if let Ok(a) = r {
                a.file_name().into_string().unwrap()
            } else {
                "".to_string()
            }
        })
        .collect();

    paths
}

// TODO: crear esto para borrar
// pub async fn delete_file(file_name: &str, ubicacion: &str) -> Result<(), Error> {
//     let filepath = get_file_path(file_name, ubicacion);
//     Ok(fs::remove_file(filepath)?)
// }

/// Realizar operacion de GET y retornar el resultado.
/// Realmente solo es para recordar.
pub fn get(con: Connection, endpoint: String) -> Result<Response, String> {
    let url = con.to_url(endpoint);
    println!("{}", url);
    let response;
    match reqwest::blocking::get(url.as_str()) {
        Ok(a) => {
            response = a;
        }
        Err(err) => {
            println!("error: {}", err);
            return Err(err.to_string());
        }
    };

    Ok(response)
}

/// Envia por la CONEXION, con el ENDPOINT especificado, la lista de
/// archivos encontrados en DIR
pub fn send_files(
    con: &Connection,
    endpoint: String,
    dir: String,
) -> Result<Response, reqwest::Error> {
    let url = con.to_url(endpoint);

    let client = reqwest::blocking::Client::new();
    let data = files_as_json(dir);

    match client
        .post(url)
        .header("content-type", "application/json")
        .body(data)
        .send()
    {
        Ok(a) => Ok(a),
        Err(err) => {
            println!("post error: {}", err);
            Err(err)
        }
    }
}

/// Convierte los archivos encontrados en DIR en un json
pub fn files_as_json(dir: String) -> String {
    let vec: Vec<String> = get_files_in_dir(dir);
    let json = serde_json::to_string(&vec);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}
