use std::{
    fs::{File, OpenOptions},
    io::Write,
    time::SystemTime,
};

use super::connection::Connection;

// TODO: quitar el blocking cuando sea posible pasar actix-web a usar tokio 1
pub type Response = reqwest::blocking::Response;
pub type Form = reqwest::blocking::multipart::Form;
pub type Client = reqwest::blocking::Client;
pub type Error = reqwest::Error;

pub fn get(url: &str) -> Result<Response, Error> {
    reqwest::blocking::get(url)
}

pub fn post_json(url: &str, data: String) -> Result<Response, Error> {
    let client = Client::new();

    client
        .post(url)
        .header("content-type", "application/json")
        .body(data)
        .send()
}

pub fn send_multipart(url: &str, form: Form) -> Result<Response, Error> {
    let client = Client::new();
    client.post(url).multipart(form).send()
}

pub fn get_as_json(url: &str) -> Result<Vec<Connection>, String> {
    match get(url) {
        Ok(respuesta) => Ok(match respuesta.text() {
            Ok(a) => serde_json::from_str(&a).unwrap(),
            Err(e) => {
                eprintln!("{}", e);
                vec![]
            }
        }),
        Err(e) => {
            return Err(format!("Error de conexion:\n{:?}", e));
        }
    }
}

/// Descarga el documento encontrado en URL en la direccion PATH.
/// Puede llamar urls externas al sistema.
pub fn download_url(url: &str, path: &str) -> Result<String, String> {
    let response = match get(url) {
        Ok(it) => it,
        Err(e) => return Err(format!("Error de conexion:\n{:?}", e)),
    };

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    let filepath = format!("{dir}/{file}", dir = path, file = fname);
    let mut dest: File = {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&filepath)
        {
            Ok(a) => a,
            Err(e) => return Err(format!("Error creando el archivo: \n {:?}", e)),
        }
    };
    let content = response.bytes().unwrap();

    match dest.write(&content) {
        Ok(_a) => Ok(format!(
            "Archivo {archivo} de {ip} descargado en {ubicacion}",
            archivo = filepath,
            ip = url,
            ubicacion = path
        )),
        Err(e) => Err(format!("Error: {:?}", e)),
    }
}

/// hacer un ""ping"" a la otra maquina
pub fn ping(url: &str) -> Result<u128, Error> {
    let start = SystemTime::now();
    // solo es para ver el tiempo de respuesta
    // esto actua como "ping"
    let _respuesta: Response = get(&url)?;
    Ok(start.elapsed().unwrap().as_millis())
}
