use serde::{Deserialize, Serialize};
use std::fmt;

use super::{
    filesystem,
    network::{self, Error, Response},
};

/// Representa una conexion, contiene ip y puerto.
#[derive(Deserialize, Serialize, Clone)]
pub struct Connection {
    pub ip: String,
    pub port: String,
}

impl Connection {
    /// Cadena base que representa a la conexion
    pub fn base_str(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// Descarga el archivo FILE_NAME en la carpeta PATH.
    /// Esto funciona unicamente dentro del sistema.
    pub fn download(&self, file_name: &str, path: &str) -> Result<String, String> {
        let url = self.to_url(format!("download/{}", file_name));
        network::download_url(&url, path)
    }

    /// Hacer ping a cada una de las conexiones y medir el tiempo de cada una
    /// retornar la ip que responda mas rapido, en caso de recibir una
    /// lista vacia, se retorna error
    fn get_conexion_mas_cercana(conexiones_posibles: &[Connection]) -> Result<&Connection, String> {
        if !conexiones_posibles.is_empty() {
            Ok(conexiones_posibles
                .iter()
                .filter_map(|con| -> Option<(&Connection, u128)> {
                    match con.ping() {
                        Ok(time) => Some((con, time)),
                        Err(_) => None,
                    }
                })
                .min_by(|con1, con2| con1.1.cmp(&con2.1))
                .unwrap()
                .0)
        } else {
            Err("Lista vacia".to_string())
        }
    }

    /// de las conexiones posibles, se elige la que responda mas
    /// rapido, y a esta se le pide el archivo FILE_NAME, este se
    /// descarga en la carpeta DIR
    pub fn get_file(&self, file_name: &str, dir: &str) -> Result<String, String> {
        let conexiones: Vec<Connection> = self.pedir_conexiones_viables(file_name).unwrap();

        if conexiones.is_empty() {
            return Err(format!(
                "No se han conseguido conexiones para el archivo {}",
                file_name
            ));
        }

        println!("{:#?}", conexiones);

        match Connection::get_conexion_mas_cercana(&conexiones) {
            Ok(url) => {
                println!("{:?}", url);
                url.download(file_name, dir)
            }
            Err(err) => {
                eprint!("Error: {}", err);
                Err(err)
            }
        }
    }

    /// Conseguir lista de conexiones de forma aleatoria.
    /// CANTIDAD es el numero de conexiones a pedir al dispatcher
    fn pedir_conexiones_aleatorias(&self, cantidad: u32) -> Result<Vec<Connection>, String> {
        let url = self.to_url(format!("get_random_connections/{}", cantidad));
        network::get_as_json(&url)
    }

    /// conseguir lista de conexiones viables que contienen un
    /// archivo especifico desde el dispatcher
    fn pedir_conexiones_viables(&self, file_name: &str) -> Result<Vec<Connection>, String> {
        let url = self.to_url(format!("get_connections/{}", file_name));
        network::get_as_json(&url)
    }

    pub fn ping(&self) -> Result<u128, network::Error> {
        network::ping(&self.to_url("ping".to_string()))
    }

    /// de las conexiones posibles, se elige la que responda mas
    /// rapido, y a esta se le envia el archivo FILE_NAME.
    /// Se piden CANTIDAD conexiones al dispatcher para probar los
    /// tiempos de respuesta.
    pub fn put_file(&self, file_path: &str, cantidad: u32) -> Result<String, String> {
        let conexiones: Vec<Connection> = self.pedir_conexiones_aleatorias(cantidad).unwrap();

        if conexiones.is_empty() {
            return Err("No se han conseguido conexiones".to_string());
        }

        println!("{:#?}", conexiones);

        match Connection::get_conexion_mas_cercana(&conexiones) {
            Ok(url) => {
                println!("{:?}", url);
                url.upload(file_path)
            }
            Err(err) => {
                eprint!("Error: {}", err);
                Err(err)
            }
        }
    }

    /// Envia por la CONEXION, con el ENDPOINT especificado, la lista de
    /// archivos encontrados en DIR
    pub fn send_files(&self, endpoint: String, dir: String) -> Result<Response, Error> {
        let url = self.to_url(endpoint);
        let data = filesystem::files_as_json(dir);

        match network::post_json(&url, data) {
            Ok(a) => Ok(a),
            Err(err) => {
                println!("post error: {}", err);
                Err(err)
            }
        }
    }

    /// Cadena como url
    pub fn to_url(&self, cad: String) -> String {
        format!("http://{}/{}", self.base_str(), cad)
    }

    pub fn upload(&self, file_path: &str) -> Result<String, String> {
        match network::Form::new().file("file", file_path) {
            Ok(form) => match network::send_multipart(&self.to_url("upload".to_string()), form) {
                Ok(_) => Ok("upload exitoso".to_string()),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({ip}:{port})", ip = self.ip, port = self.port)
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({ip}:{port})", ip = self.ip, port = self.port)
    }
}
