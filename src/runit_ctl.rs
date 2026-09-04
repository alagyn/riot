use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::sync::LazyLock;

use regex::Regex;

static STATUS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?P<status>(run)|(down)|(fail)): (?P<name>[/\w-]+):( \(pid (?P<pid>\d+)\))? (?P<runtime>\d+)s(, (?P<info>[\w\s,]+))?",
).unwrap()
});

use crate::service::ServiceStatus;

pub fn get_status(service_dir: &PathBuf) -> Result<ServiceStatus, String> {
    let stat_file = service_dir.join("supervise").join("stat");

    if !stat_file.is_file() {
        return Err(format!("{} is not a file", stat_file.to_str().unwrap()));
    }

    let status = match std::fs::read_to_string(&stat_file) {
        Ok(text) => text,
        Err(err) => {
            return Err(format!(
                "Error reading {}: {}",
                stat_file.to_str().unwrap(),
                err.to_string()
            ));
        }
    };

    let status = match status.trim() {
        "run" => ServiceStatus::Up,
        "down" => ServiceStatus::Down,
        _ => {
            println!("Unknown status: '{}'", &status);
            ServiceStatus::Unknown
        }
    };

    Ok(status)
}

pub fn send_signal(service_dir: &PathBuf, signal: u8) {
    let control_file = service_dir.join("supervise").join("control");

    println!(
        "Sending signal {} to {}",
        String::from_utf8_lossy(&[signal]),
        control_file.to_str().unwrap()
    );

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .read(false)
        .create(false)
        .open(control_file)
        .unwrap();

    f.write_all(&[signal]).unwrap();
}
