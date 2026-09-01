use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::responses::ServiceStatusRes;
use crate::service::ServiceStatus;

pub mod auth;
pub mod responses;
pub mod runit_ctl;
pub mod service;

use service::Service;

pub const VERSION: &'static str = "v0.1.0";

pub struct Riot {
    // Service directory that is actively maintained by runsvdir
    pub svdir: PathBuf,
    // Main staging dir
    pub staging_dir: PathBuf,
    // Generated riot config data
    pub config_dir: PathBuf,
    // Directory of installed services. Enabled services are symlinked to svdir
    pub install_dir: PathBuf,
    // List of services
    pub services: Vec<Service>,
}

impl Riot {
    pub fn new(svdir: PathBuf, staging_dir: PathBuf) -> Result<Riot, String> {
        ensure_directory_exists(&svdir)?;
        ensure_directory_exists(&staging_dir)?;

        let config_dir = staging_dir.clone().join("config");
        let install_dir = staging_dir.clone().join("install");

        ensure_directory_exists(&config_dir)?;
        ensure_directory_exists(&install_dir)?;

        let installed_services = list_service_dir(&install_dir);
        let enabled_services: HashSet<String> = list_service_dir(&svdir).iter().cloned().collect();

        let mut services: Vec<Service> = Vec::new();

        for name in installed_services {
            let enabled = enabled_services.contains(&name);

            services.push(Service { name, enabled });
        }

        Ok(Riot {
            svdir,
            staging_dir,
            config_dir,
            install_dir,
            services,
        })
    }

    pub fn list_services(&self) -> responses::ServiceList {
        let _out_services: Vec<ServiceStatusRes> =
            self.services.iter().map(|x| x.check_status()).collect();

        responses::ServiceList {
            services: _out_services,
        }
    }
}

fn list_service_dir(dir: &PathBuf) -> Vec<String> {
    let entries: fs::ReadDir = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => panic!("Failed to list svdir"),
    };

    let mut svcs: Vec<String> = entries
        .filter_map(|x| match x {
            Ok(entry) => Some(
                entry
                    .file_name()
                    .into_string()
                    .expect("Failed to parse filename"),
            ),
            Err(_) => None,
        })
        .collect();

    svcs.sort_unstable();
    svcs
}

fn ensure_directory_exists(dir: &PathBuf) -> Result<(), String> {
    if dir.is_dir() {
        return Ok(());
    }

    if dir.exists() {
        return Err(format!(
            "Cannot make directory '{}' a file exists with same name",
            dir.to_str().unwrap()
        ));
    }

    match std::fs::create_dir_all(dir) {
        Ok(_) => Ok(()),
        Err(msg) => Err(msg.to_string()),
    }
}
