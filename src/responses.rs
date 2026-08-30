use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatusRes {
    pub name: String,
    pub uptime: String,
    pub enabled: bool,
    pub status: crate::service::ServiceStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceList {
    pub services: Vec<ServiceStatusRes>,
}
