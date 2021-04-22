use serde::{Deserialize, Serialize};
use std::fmt;

/// Representa una conexion, contiene ip y puerto.
#[derive(Deserialize, Serialize, Clone)]
pub struct Connection {
    pub ip: String,
    pub port: String,
}

impl Connection {
    pub fn to_string(&self, cad: String) -> String {
        format!("http://{}:{}/{}", self.ip, self.port, cad)
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({ip}:{port})", ip = self.ip, port = self.port)
    }
}