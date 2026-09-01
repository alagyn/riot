use serde::{Deserialize, Serialize};

use crate::{responses::ServiceStatusRes, runit_ctl};

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
    pub fn check_status(&self) -> ServiceStatusRes {
        if self.enabled {
            match runit_ctl::get_status(&self.name) {
                Ok(status) => status,
                Err(msg) => {
                    println!("Failed to get status for service {}: {}", self.name, msg);
                    ServiceStatusRes {
                        name: self.name.clone(),
                        uptime: String::from("TODO"),
                        enabled: self.enabled,
                        status: ServiceStatus::Unknown,
                    }
                }
            }
        } else {
            ServiceStatusRes {
                name: self.name.clone(),
                uptime: String::from("TODO"),
                enabled: self.enabled,
                status: ServiceStatus::Unknown,
            }
        }
    }

    pub fn start(&self) -> Result<ServiceStatusRes, String> {
        // TODO
        panic!("unimplemented")
    }

    pub fn stop(&self) -> Result<ServiceStatusRes, String> {
        // TODO
        panic!("unimplemented")
    }
}
