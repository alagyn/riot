use serde::{Deserialize, Serialize};

use crate::runit_ctl;

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

impl Service {
    pub fn check_status(&self) -> Result<ServiceStatus, String> {
        runit_ctl::get_status(&self.name)
    }
}
