use std::sync::Mutex;
use std::time::Instant;
use sysinfo::System;

pub struct AppState {
    pub sys: Mutex<System>,
    pub last_db_save: Mutex<Instant>,
}
