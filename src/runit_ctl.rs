use std::process;

use regex::Regex;

// const STATUS_RE = Regex::new(r"(?P<status>(run)|(down)|(fail)): (?P<name>[/\w-]+):( \(pid (?P<pid>\d+)\))? (?P<runtime>\d+)s(, (?P<info>[\w\s,]+))?");

use crate::service::ServiceStatus;

pub fn get_status(service: &str) -> Result<ServiceStatus, String> {
    let args: Vec<String> = vec![
        String::from("sv"),
        String::from("status"),
        service.to_string(),
    ];

    match process::Command::new("sv").args(args).output() {
        Ok(output) => Ok(ServiceStatus::Disabled),
        Err(msg) => Err(msg.to_string()),
    }
}
