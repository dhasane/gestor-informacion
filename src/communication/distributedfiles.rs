#![allow(dead_code)]
use chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::communication::connection::Connection;

pub static STANDBY_THRESHOLD: u128 = 30;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Copy, Debug)]
pub enum State {
    Connected,
    Disconnected,
    Standby,
}

/// Contiene la conexion y el conjunto de archivos que se encuentran en esta.
#[derive(Deserialize, Serialize, Clone)]
pub struct DistributedFiles {
    pub archivos: Vec<String>,
    pub conexion: Connection,
    pub state: State,
    state_change: DateTime<chrono::Utc>,
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

    pub fn duration_since_state_change(&self) -> Duration {
        chrono::Utc::now().signed_duration_since(self.state_change)
    }

    pub fn new(con: Connection, files: Vec<String>) -> DistributedFiles {
        DistributedFiles {
            conexion: con,
            archivos: files,
            state: State::Connected,
            state_change: chrono::Utc::now(),
        }
    }

    /// Prueba la conexion y modifica state.
    /// Retorna un bool significando si ocurrio un cambio
    pub fn test_connection(&mut self) -> bool {
        if self.state == State::Disconnected {
            return false;
        }
        // print!("{}", self);
        match self.conexion.ping() {
            Ok(time) if self.state == State::Connected && time >= STANDBY_THRESHOLD => {
                // println!("1 >> {:?} == {} : {}", self.state, self.state_change, time);
                self.state = State::Standby;
                self.state_change = chrono::Utc::now();
                true
            }
            Ok(time) if self.state == State::Standby && time < STANDBY_THRESHOLD => {
                // println!("2 >> {:?} == {} : {}", self.state, self.state_change, time);
                self.state = State::Connected;
                self.state_change = chrono::Utc::now();
                true
            }
            Ok(_time) => false,
            Err(_) => {
                // println!("Err >> {:?} == {}", self.state, self.state_change);
                self.state = State::Disconnected;
                self.state_change = chrono::Utc::now();
                true
            }
        }
    }
}

impl fmt::Display for DistributedFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}):{:?}", self.conexion.base_str(), self.archivos)
    }
}
