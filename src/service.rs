use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ServiceStatus {
    Unknown,
    Down,
    WantDown,
    Up,
    WantUp,
}

pub struct Service {
    pub name: String,
    pub enabled: bool,
}
