#![allow(dead_code)]

use std::collections::HashMap;

use crate::communication::{connection::Connection, distributedfiles::DistributedFiles};

pub struct FileList {
    archivos: Vec<DistributedFiles>,
}

impl FileList {
    /// Agrega una conexion y sus archivos al registro.
    /// Puede que haya una manera mas facil, pero de momento esto parece
    /// servir.
    pub fn add_or_replace_connection(&mut self, con: Connection, files: Vec<String>) {
        let accion = format!("{} -> {:?}", con.base_str(), files);
        let dist_file = DistributedFiles {
            conexion: con,
            archivos: files,
        };
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

        self.print();
    }

    pub fn create() -> FileList {
        let vect: Vec<DistributedFiles> = vec![];
        FileList { archivos: vect }
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
    /// Retorna una copia de la lista conexiones.
    pub fn get_connections_by_filename(&self, nombre: &str) -> Vec<Connection> {
        let archivos: &Vec<DistributedFiles> = &self.get_files();
        archivos
            .iter()
            .filter(|&df| df.archivos.iter().any(|f| -> bool { f == nombre }))
            .map(|f| -> Connection { f.conexion.clone() })
            .collect()
    }

    /// Conseguir todas las conexiones que no contienen un archivo especifico.
    /// Retorna una copia de la lista conexiones.
    pub fn get_connections_without_filename(&self, nombre: &str) -> Vec<Connection> {
        let archivos: &Vec<DistributedFiles> = &self.get_files();

        archivos
            .iter()
            .filter(|&df| !df.archivos.iter().any(|f| -> bool { f == nombre }))
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

    /// Conseguir todas las conexiones que no contienen un archivo especifico.
    /// Retorna una copia de la lista conexiones.
    pub fn get_number_of_files(&self) -> Vec<(String, u64)> {
        let archivos_dist: &Vec<DistributedFiles> = &self.get_files();
        let mut numero_archivos: HashMap<String, u64> = HashMap::new();

        for archivo in archivos_dist {
            for a in &archivo.archivos {
                *numero_archivos.entry(a.to_string()).or_insert(0) += 1;
            }
        }

        numero_archivos
            .into_iter()
            .map(|(key, value)| (key, value))
            .collect()
    }

    pub fn print(&self) {
        for distrib in &self.archivos {
            println!("conexion : {}", distrib.conexion);
            for archivo in &distrib.archivos {
                println!("\t- {}", archivo);
            }
        }
    }

    pub fn size(&self) -> u64 {
        self.archivos.len() as u64
    }
}
