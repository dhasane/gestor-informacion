#![allow(dead_code)]
use serde::{Deserialize, Serialize};

use crate::communication::connection::Connection;

/// Contiene la conexion y el conjunto de archivos que se encuentran en esta.
#[derive(Deserialize, Serialize, Clone)]
pub struct DistributedFiles {
    pub conexion: Connection,
    pub archivos: Vec<String>,
}

impl PartialEq for DistributedFiles {
    fn eq(&self, other: &Self) -> bool {
        self.conexion.ip == other.conexion.ip && self.conexion.port == other.conexion.port
    }
}

impl PartialEq<DistributedFiles> for &DistributedFiles {
    fn eq(&self, other: &DistributedFiles) -> bool {
        self.conexion.ip == other.conexion.ip && self.conexion.port == other.conexion.port
    }
}

impl DistributedFiles {
    pub fn comp(&self, ip: &str, port: &str) -> bool {
        self.conexion.ip == ip && self.conexion.port == port
    }

    pub fn contains_file(&self, filename: String) -> bool {
        self.archivos.contains(&filename)
    }
}
