use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::OpenOptions;
use std::io::copy;
use std::time::SystemTime;

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

    /// Cadena como url
    pub fn to_url(&self, cad: String) -> String {
        format!("http://{}/{}", self.base_str(), cad)
    }

    /// hacer un ""ping"" a la otra maquina
    pub fn ping(&self) -> Result<u128, reqwest::Error> {
        let start = SystemTime::now();
        let url = self.to_url("ping".to_string());
        // solo es para ver el tiempo de respuesta
        // esto actua como "ping"
        let _respuesta: reqwest::blocking::Response = reqwest::blocking::get(url)?;
        Ok(start.elapsed().unwrap().as_millis())
    }

    /// conseguir lista de conexiones viables que contienen un
    /// archivo especifico desde el dispatcher
    fn pedir_conexiones_viables(&self, file_name: &str) -> Result<Vec<Connection>, String> {
        let url = self.to_url(format!("get_connections/{}", file_name));

        let respuesta = match reqwest::blocking::get(url) {
            Ok(it) => it,
            Err(e) => {
                return Err(format!("Error de conexion:\n{:?}", e));
            }
        };

        Ok(match respuesta.text() {
            Ok(a) => serde_json::from_str(&a).unwrap(),
            Err(e) => {
                eprintln!("{}", e);
                vec![]
            }
        })
    }

    /// Conseguir lista de conexiones de forma aleatoria.
    /// CANTIDAD es el numero de conexiones a pedir al dispatcher
    fn pedir_conexiones_aleatorias(&self, cantidad: u32) -> Result<Vec<Connection>, String> {
        let url = self.to_url(format!("get_random_connections/{}", cantidad));

        let respuesta = match reqwest::blocking::get(url) {
            Ok(it) => it,
            Err(e) => {
                return Err(format!("Error de conexion:\n{:?}", e));
            }
        };

        Ok(match respuesta.text() {
            Ok(a) => serde_json::from_str(&a).unwrap(),
            Err(e) => {
                eprintln!("{}", e);
                vec![]
            }
        })
    }

    /// Hacer ping a cada una de las conexiones y medir el tiempo de cada una
    /// retornar la ip que responda mas rapido, en caso de recibir una
    /// lista vacia, se retorna error
    fn get_conexion_mas_cercana(conexiones_posibles: &[Connection]) -> Result<&Connection, String> {
        if !conexiones_posibles.is_empty() {
            Ok(conexiones_posibles
                .iter()
                .map(|con| (con, con.ping().unwrap()))
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

    /// Descargarga el archivo FILE_NAME en la carpeta PATH
    pub fn download(&self, file_name: &str, path: &str) -> Result<String, String> {
        let url = self.to_url(format!("download/{}", file_name));

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

            let filepath = format!("{dir}/{file}", dir = path, file = fname);
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
            Ok(_a) => Ok(format!(
                "Archivo {archivo} de {ip} descargado en {ubicacion}",
                archivo = file_name,
                ip = self,
                ubicacion = path
            )),
            Err(e) => Err(format!("Error: {:?}", e)),
        }
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

    // TODO: hacer el equivalente de get_file
    pub fn upload(&self, file_path: &str) -> Result<String, String> {
        match multipart::Form::new().file("file", file_path) {
            Ok(form) => {
                let client = reqwest::blocking::Client::new();
                match client
                    .post(self.to_url("upload".to_string()))
                    .multipart(form)
                    .send()
                {
                    Ok(_) => Ok("upload exitoso".to_string()),
                    Err(e) => Err(e.to_string()),
                }
            }
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
