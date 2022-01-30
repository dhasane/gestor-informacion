#![allow(dead_code)]

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
};

use crate::communication::{connection::Connection, distributedfiles::DistributedFiles};

use super::distributedfiles::State;

#[derive(Serialize, Deserialize)]
pub struct FileList {
    archivos: Vec<DistributedFiles>,
    modified: bool,
}

impl FileList {
    /// Agrega una conexion y sus archivos al registro.
    /// Puede que haya una manera mas facil, pero de momento esto parece
    /// servir.
    pub fn add_or_replace_connection(&mut self, con: Connection, files: Vec<String>) {
        let accion = format!("{} -> {:?}", con.base_str(), files);
        let dist_file = DistributedFiles::new(con, files);
        if let Some(pos) = self
            .archivos
            .iter()
            .position(|a| -> bool { a == dist_file })
        {
            println!("reemplazando conexion: {} ", accion);
            let _got = std::mem::replace(&mut self.archivos[pos], dist_file);
        } else {
            println!("nueva conexion: {} ", accion);
            self.archivos.push(dist_file);
        }

        self.modified = true;
    }

    pub fn create() -> FileList {
        let vect: Vec<DistributedFiles> = vec![];
        FileList {
            archivos: vect,
            modified: false,
        }
    }

    /// Conseguir todas las conexiones que contienen un archivo especifico.
    /// Retorna una copia de la lista conexiones.
    pub fn get_connections(&self) -> Vec<Connection> {
        let archivos: &Vec<DistributedFiles> = &self.get_files();
        archivos
            .iter()
            .map(|f| -> Connection { f.conexion.clone() })
            .collect()
    }

    /// Conseguir todas las conexiones que contienen un archivo especifico.
    /// En caso de que with sea false, se retnornan las conexiones que no tengan el archivo
    /// Retorna una copia de la lista conexiones.
    pub fn get_connections_by_filename(&self, nombre: &str, with: bool) -> Vec<Connection> {
        let archivos: &Vec<DistributedFiles> = &self.get_files();
        archivos
            .iter()
            .filter(|&df| with == df.archivos.iter().any(|f| -> bool { f == nombre }))
            .map(|f| -> Connection { f.conexion.clone() })
            .collect()
    }

    /// Conseguir todos los archivos en una conexion especifica.
    /// En caso de no encontrar la conexion, retorna un vector vacio.
    /// Retorna una copia de la lista de archivos de la conexion.
    pub fn get_filenames_by_connection(
        &self,
        ip: &str,
        port: &str,
    ) -> Result<&Vec<String>, String> {
        let archivos: &Vec<DistributedFiles> = self.get_files();
        match archivos.iter().find(|&df| df.comp(ip, port)) {
            Some(f) => Ok(&f.archivos),
            None => Err("Conexion no encontrada".to_string()),
        }
    }

    /// Retorna una copia de la lista de archivos
    pub fn get_files(&self) -> &Vec<DistributedFiles> {
        &self.archivos
    }

    /// Retorna una lista de cada archivo en el sistema y la cantidad
    /// de veces que aparece
    pub fn get_number_of_files(&self, count_disconected: bool) -> Vec<(String, u64)> {
        let archivos_dist: &Vec<DistributedFiles> = &self.get_files();
        let mut numero_archivos: HashMap<String, u64> = HashMap::new();

        for distrib_file in archivos_dist {
            // TODO: maybe this should be used better
            if count_disconected || distrib_file.state != State::Disconnected {
                for file_name in &distrib_file.archivos {
                    *numero_archivos.entry(file_name.to_string()).or_insert(0) += 1;
                }
            }
        }

        numero_archivos
            .into_iter()
            .map(|(key, value)| (key, value))
            .collect()
    }

    pub fn load(path: &str) -> FileList {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);

                serde_json::from_reader(reader).unwrap()
            }
            Err(_e) => {
                println!("Config file not found");
                FileList::create()
            }
        }
    }

    pub fn print(&self) {
        for distrib in &self.archivos {
            println!("conexion : {} [{:?}]", distrib.conexion, distrib.state);
            for archivo in &distrib.archivos {
                println!("\t- {}", archivo);
            }
        }
    }

    pub fn size(&self) -> u64 {
        self.archivos.len() as u64
    }

    pub fn store(&mut self, path: &str) {
        // -> Result<String, String> {
        if !self.modified {
            return; // Ok("nothing has been modified".to_string());
        }
        self.print();
        self.modified = false;
        let mut dest = {
            match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)
            {
                Ok(a) => a,
                Err(e) => {
                    // return Err(format!("Error creando el archivo: \n {:?}", e))
                    println!("Error creating file: \n {:?}", e);
                    return;
                }
            }
        };
        println!(
            "{}",
            match dest.write(&serde_json::to_vec(self).unwrap()) {
                Ok(_a) => format!("File {ubicacion} writen", ubicacion = path),
                Err(e) => format!("Error: {:?}", e),
            }
        );
    }

    /// Revisa los tiempos de respuesta de cada conexion, en caso de
    /// que se desconecte, al pasar un tiempo predeterminado, elimina
    /// la conexion de la lista
    pub fn test_connections(&mut self) {
        // TODO: check this through configuration
        // TODO: could be better to move configuration values to an external file
        let ttl = Duration::hours(3);
        // let ttl = Duration::seconds(3);

        // TODO: make this nicer
        let mut index = 0;
        let mut remove: Vec<usize> = Vec::new();

        for file in &mut self.archivos {
            if file.test_connection() {
                self.modified = true;
            }
            if file.state == State::Disconnected && file.duration_since_state_change() > ttl {
                // remove.append(file)
                println!("remove {} from list", file.conexion);
                remove.push(index);
            }
            index += 1;
        }

        if !remove.is_empty() {
            self.modified = true;
        }
        for r in remove {
            self.archivos.remove(r);
        }
    }
}
