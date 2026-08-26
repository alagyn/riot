use std::fs;
use std::path::PathBuf;

pub mod service;

pub struct Riot {
    pub svdir: PathBuf,
    pub staging_dir: PathBuf,
    pub services: Vec<String>,
}

impl Riot {
    pub fn new(svdir: PathBuf, staging_dir: PathBuf) -> Riot {
        let entries: fs::ReadDir = match fs::read_dir(&svdir) {
            Ok(entries) => entries,
            Err(_) => panic!("Failed to list svdir"),
        };

        let services: Vec<String> = entries
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

        Riot {
            svdir,
            staging_dir,
            services,
        }
    }
}
