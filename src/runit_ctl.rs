use std::process;
use std::sync::LazyLock;

use regex::Regex;

static STATUS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?P<status>(run)|(down)|(fail)): (?P<name>[/\w-]+):( \(pid (?P<pid>\d+)\))? (?P<runtime>\d+)s(, (?P<info>[\w\s,]+))?",
).unwrap()
});

use crate::responses::ServiceStatusRes;
use crate::service::ServiceStatus;

pub fn get_status(service: &str) -> Result<ServiceStatusRes, String> {
    let args: Vec<String> = vec![String::from("status"), service.to_string()];

    let output = match process::Command::new("sv").args(args).output() {
        Ok(output) => output,
        Err(msg) => return Err(msg.to_string()),
    };

    let Some(m) = STATUS_RE.captures(std::str::from_utf8(&output.stdout).unwrap()) else {
        return Err(String::from("Failed to parse sv output"));
    };

    panic!("unimplemented");

    let status = match &m["status"] {
        "run" => ServiceStatus::Up,
        "down" => ServiceStatus::Down,
        _ => ServiceStatus::Unknown,
    };
}
