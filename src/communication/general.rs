#![allow(dead_code)]
use std::fs;

use reqwest::blocking::Response;
use std::fs::OpenOptions;
use std::io::copy;
use std::time::SystemTime;

use crate::communication::connection::Connection;

/// Conseigue los archivos que se encuentran en UBICACION y retorna una lista de strings
pub fn get_files(dir: String) -> Vec<String> {
    let paths: Vec<String> = fs::read_dir(dir)
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

// TODO: crear esto para borrar
// pub async fn delete_file(file_name: &str, ubicacion: &str) -> Result<(), Error> {
//     let filepath = get_file_path(file_name, ubicacion);
//     Ok(fs::remove_file(filepath)?)
// }

/// Realizar operacion de GET y retornar el resultado.
/// Realmente solo es para recordar.
pub fn get(con: Connection, endpoint: String) -> Result<Response, ()> {
    let url = con.to_string(endpoint);
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

/// Envia por la CONEXION, con el ENDPOINT especificado, la lista de
/// archivos encontrados en DIR
pub fn send_files(
    con: &Connection,
    endpoint: String,
    dir: String,
) -> Result<Response, reqwest::Error> {
    let url = con.to_string(endpoint);

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
    let vec: Vec<String> = get_files(dir);
    let json = serde_json::to_string(&vec);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}

/// conseguir lista de direcciones viables que contienen un
/// archivo especifico desde el dispatcher
pub fn pedir_ips_viables(
    con_dispatcher: &Connection,
    file_name: &str,
) -> Result<Vec<Connection>, ()> {
    let url = con_dispatcher.to_string(format!("getdirs/{}", file_name));

    let respuesta = match reqwest::blocking::get(url) {
        Ok(it) => it,
        Err(e) => {
            println!("Error de conexion:\n{:?}", e);
            return Err(());
        }
    };

    let texto = respuesta.text();

    let json = match texto {
        Ok(a) => a,
        Err(e) => {
            println!("{:?}", e);
            "".to_string()
        }
    };
    let ret: Vec<Connection> = if json != "" {
        serde_json::from_str(&json).unwrap()
    } else {
        vec![]
    };
    Ok(ret)
}

/// hacer un ""ping"" a la otra maquina
pub fn ping(con: &Connection) -> Result<u128, reqwest::Error> {
    let start = SystemTime::now();
    let url = con.to_string(format!("ping"));
    // solo es para ver el tiempo de respuesta
    // esto actua como "ping"
    let _respuesta: reqwest::blocking::Response = reqwest::blocking::get(url)?;
    Ok(start.elapsed().unwrap().as_millis())
}

/// Hacer ping a cada una de las ips y medir el tiempo de cada una
/// retornar la ip que responda mas rapido
fn get_conexion_mas_cercana(conexiones_posibles: Vec<Connection>) -> Connection {
    let mut cons = conexiones_posibles
        .iter()
        .map(|con| (con, ping(con).unwrap()));
    // TODO: seria chevere sacar el menor desde aca con min o algo asi

    let algo = cons.next().unwrap();

    let mut ret: &Connection = algo.0;
    let mut min: u128 = algo.1;

    for con in cons {
        if con.1 < min {
            ret = con.0;
            min = con.1;
        }
    }
    ret.to_owned()
}

pub fn get_file(con_dispatcher: &Connection, file_name: String, dir: String) -> String {
    let ips: Vec<Connection> = pedir_ips_viables(con_dispatcher, &file_name).unwrap();

    if ips.is_empty() {
        return format!(
            "No se han conseguido direcciones para el archivo {}",
            file_name
        );
    }

    let ips_viables: Vec<Connection> = ips.iter().map(|f| -> Connection { f.clone() }).collect();

    println!("{:#?}", ips_viables);

    let url = get_conexion_mas_cercana(ips_viables);

    println!("{:?}", url);

    match download(url, file_name, dir) {
        Ok(_) => {
            format!("Archivo descargado")
        }
        Err(e) => {
            format!("{}", e)
        }
    }
}

pub fn download(con: Connection, nombre_archivo: String, ubicacion: String) -> Result<(), String> {
    let url = con.to_string(format!("file/{}", nombre_archivo));

    // TODO: quitar el blocking cuando sea posible pasar actix-web a usar tokio 1
    let response = match reqwest::blocking::get(url) {
        Ok(it) => it,
        Err(e) => return Err(format!("Error de conexion:\n{:?}", e)),
    };

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        let filepath = format!("{dir}/{file}", dir = ubicacion, file = fname);
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filepath)
        {
            Ok(a) => a,
            Err(e) => return Err(format!("Error creando el archivo: \n {:?}", e)),
        }
    };
    let content = response.text().unwrap();
    match copy(&mut content.as_bytes(), &mut dest) {
        Ok(_a) => {
            println!(
                "Descargando archivo {archivo} de {ip} en {ubicacion}",
                archivo = nombre_archivo,
                ip = con,
                ubicacion = ubicacion
            );

            Ok(())
        }
        Err(e) => return Err(format!("Error: {:?}", e)),
    }
}
