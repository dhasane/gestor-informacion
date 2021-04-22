#![allow(dead_code)]

use crate::communication::{connection::Connection, distributedfiles::DistributedFiles};

pub struct FileList {
    archivos: Vec<DistributedFiles>,
}

impl FileList {
    pub fn create() -> FileList {
        let vect: Vec<DistributedFiles> = vec![];
        FileList { archivos: vect }
    }

    pub fn print(&self) {
        for distrib in &self.archivos {
            println!("conexion : {}", distrib.conexion);
            for archivo in &distrib.archivos {
                println!("\t- {}", archivo);
            }
        }
    }

    /// Agrega una conexion y sus archivos al registro.
    /// Puede que haya una manera mas facil, pero de momento esto parece
    /// servir.
    pub fn agregar_conexion(&mut self, ip: &str, port: &str, files: Vec<String>) {
        let con = DistributedFiles {
            conexion: Connection {
                ip: ip.to_string(),
                port: port.to_string(),
            },
            archivos: files,
        };
        self.archivos.push(con);
        self.print();
    }

    /// Retorna una copia del registro de archivos.
    pub fn clone(&self) -> Vec<DistributedFiles> {
        self.archivos.to_vec()
    }

    /// Conseguir todos los archivos en una conexion especifica.
    /// En caso de no encontrar la conexion, retorna un vector vacio.
    /// Retorna una copia de la lista de archivos de la conexion.
    pub fn get_filenames_by_connection(&self, ip: &str, port: &str) -> Vec<String> {
        let archivos: &Vec<DistributedFiles> = &self.archivos;
        match archivos.iter().find(|&df| df.comp(ip, port)) {
            Some(f) => f.archivos.clone(),
            None => vec![],
        }
    }

    /// Conseguir todas las conexiones que contienen un archivo especifico.
    /// Retorna una copia de la lista conexiones.
    pub fn get_connections_by_filename(&self, nombre: &str) -> Vec<Connection> {
        let archivos: &Vec<DistributedFiles> = &self.archivos;
        archivos
            .iter()
            .filter(|&df| df.archivos.iter().any(|f| -> bool { f == nombre }))
            .map(|f| -> Connection { f.conexion.clone() })
            .collect()
    }
}
