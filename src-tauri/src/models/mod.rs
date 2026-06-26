// src/models/mod.rs
use std::time::Instant;
pub mod hardware;
pub mod historico;

use std::sync::Mutex;
use sysinfo::System;

pub struct AppState {
    pub sys: Mutex<System>,
    pub last_db_save: Mutex<Instant>,
}
