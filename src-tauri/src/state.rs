use std::sync::Mutex;
use std::time::Instant;
use sysinfo::{Disks, Networks, System};

pub struct AppState {
    pub sys: Mutex<System>,
    pub disks: Mutex<Disks>,
    pub networks: Mutex<Networks>,
    pub last_db_save: Mutex<Instant>,
}
