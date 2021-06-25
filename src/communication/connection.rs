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
    /// Cadena base
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
        let url = self.to_url(format!("ping"));
        // solo es para ver el tiempo de respuesta
        // esto actua como "ping"
        let _respuesta: reqwest::blocking::Response = reqwest::blocking::get(url)?;
        Ok(start.elapsed().unwrap().as_millis())
    }

    /// conseguir lista de direcciones viables que contienen un
    /// archivo especifico desde el dispatcher
    fn pedir_ips_viables(&self, file_name: &str) -> Result<Vec<Connection>, ()> {
        let url = self.to_url(format!("get_connections/{}", file_name));

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

    /// Hacer ping a cada una de las ips y medir el tiempo de cada una
    /// retornar la ip que responda mas rapido, en caso de recibir una
    /// lista vacia, se retorna error
    fn get_conexion_mas_cercana(
        conexiones_posibles: Vec<Connection>,
    ) -> Result<Connection, String> {
        if conexiones_posibles.len() > 0 {
            Ok(conexiones_posibles
                .iter()
                .map(|con| (con.to_owned(), con.ping().unwrap()))
                .min_by(|con1, con2| con1.1.cmp(&con2.1))
                .unwrap()
                .0)
        } else {
            Err("Lista vacia".to_string())
        }
    }

    pub fn get_file(&self, file_name: String, dir: String) -> String {
        let ips: Vec<Connection> = self.pedir_ips_viables(&file_name).unwrap();

        if ips.is_empty() {
            return format!(
                "No se han conseguido direcciones para el archivo {}",
                file_name
            );
        }

        let ips_viables: Vec<Connection> =
            ips.iter().map(|f| -> Connection { f.clone() }).collect();

        println!("{:#?}", ips_viables);

        match Connection::get_conexion_mas_cercana(ips_viables) {
            Ok(url) => {
                println!("{:?}", url);

                match url.download(file_name, dir) {
                    Ok(_) => {
                        format!("Archivo descargado")
                    }
                    Err(e) => {
                        format!("{}", e)
                    }
                }
            }
            Err(err) => {
                eprint!("Error: {}", err);
                err
            }
        }
    }

    pub fn download(&self, file_name: String, ubicacion: String) -> Result<(), String> {
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
                    archivo = file_name,
                    ip = self,
                    ubicacion = ubicacion
                );

                Ok(())
            }
            Err(e) => return Err(format!("Error: {:?}", e)),
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
