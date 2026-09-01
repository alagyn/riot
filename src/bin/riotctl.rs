use std::{env, path::PathBuf};

use riot;
use riot::auth::load_riotctl_config;
use riot::responses::ServiceList;

type CmdFunc = fn(&str, reqwest::blocking::Client, &[String]) -> ();

struct Command {
    name: &'static str,
    args: &'static str,
    help: &'static str,
    func: CmdFunc,
}

fn usage(cmds: &Vec<Command>) {
    println!("riotctl {}", riot::VERSION);
    println!("USAGE:");
    println!("riotctl [cmd] <args...>");
    println!("Commands:");
    for cmd in cmds {
        println!("  {} {}", &cmd.name, &cmd.args);
        println!("    {}", &cmd.help);
    }
}

fn main() {
    let mut cmds: Vec<Command> = Vec::new();

    cmds.push(Command {
        name: "push",
        args: "[filename]",
        help: "Send a service script",
        func: send_service,
    });

    cmds.push(Command {
        name: "ls",
        args: "",
        help: "List services",
        func: list_services,
    });

    cmds.push(Command {
        name: "rm",
        args: "[service]",
        help: "Stop and remove a service",
        func: rm_service,
    });

    cmds.push(Command {
        name: "up",
        args: "[services...]",
        help: "Start a service",
        func: service_up,
    });

    cmds.push(Command {
        name: "down",
        args: "[services...]",
        help: "Stop a service",
        func: service_down,
    });

    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        usage(&cmds);
        return;
    }

    if args[1] == "help" {
        usage(&cmds);
        return;
    }

    // TODO load from env var?

    let config_file: PathBuf;

    if let Ok(path) = env::var("RIOT_CONFIG") {
        config_file = path.into();
    } else {
        config_file = match env::consts::OS {
            "linux" => match env::var("HOME") {
                Ok(val) => PathBuf::from(val)
                    .join(".config")
                    .join("bdd")
                    .join("riot")
                    .join("riot.json"),
                Err(_) => "riot.json".into(),
            },
            "windows" => match env::var("APPDATA") {
                Ok(val) => PathBuf::from(val)
                    .join("bdd")
                    .join("riot")
                    .join("riot.json"),
                Err(_) => "riot.json".into(),
            },
            _ => "riot.json".into(),
        }
    }

    let config = match load_riotctl_config(&config_file) {
        Ok(x) => x,
        Err(msg) => {
            println!("Error loading config: {}", msg);
            return;
        }
    };

    let base_url = String::from("https://") + &config.server_name;

    let client = reqwest::blocking::Client::builder()
        .tls_certs_only(reqwest::Certificate::from_pem(
            &config.server_cert.into_bytes(),
        ))
        .identity(reqwest::Identity::from_pem(&config.client_key.into_bytes()).unwrap())
        .build()
        .unwrap();

    for cmd in &cmds {
        if cmd.name == args[1] {
            (cmd.func)(base_url.as_str(), client, &args[2..]);
            return;
        }
    }

    usage(&cmds);
}

// fn check_unauthorized(
//     res: reqwest::Result<reqwest::Response>,
// ) -> Result<reqwest::Response, String> {
//     match res {
//         Ok(res) => Ok(res),
//         Err(err) => {
//             if err.is_request() {
//                 if let Some(source) = err.source() {
//                     let source = source.downcast_ref::<reqwest::Error>().unwrap();
//                 }
//             } else {
//                 Err(format!("{:?}", err))
//             }
//         }
//     }
// }

fn send_service(base_url: &str, client: reqwest::blocking::Client, args: &[String]) {
    if args.len() != 1 {
        println!("USAGE:");
        println!("riotctl push [service.sh]");
        return;
    }

    let service_file: PathBuf = args[0].clone().into();
    if !service_file.is_file() {
        println!("{} is not a file", &service_file.to_str().unwrap());
        return;
    }

    let body = std::fs::read(&service_file).expect("Failed to read service file");
    let endpoint =
        String::from(base_url) + "/services/" + service_file.file_stem().unwrap().to_str().unwrap();

    let res = client
        .post(endpoint)
        .body(body)
        .send()
        .expect("Failed to push service");

    if res.status() != reqwest::StatusCode::CREATED {
        panic!("Failed to push service {}", res.text().unwrap());
    }

    // TODO
    println!("Created");
}

fn list_services(base_url: &str, client: reqwest::blocking::Client, _args: &[String]) {
    let endpoint = String::from(base_url) + "/services";

    let res = match client.get(endpoint).send() {
        Ok(res) => res,
        Err(msg) => {
            println!("Failed to list services {:?}", msg);
            return;
        }
    };

    if res.status() != reqwest::StatusCode::OK {
        // TODO?
        println!("Failed to list services");
        return;
    }

    let services: ServiceList = res.json().unwrap();
    if !services.services.is_empty() {
        println!("Services:");
        for x in services.services {
            println!("{:?}", x);
        }
    } else {
        println!("No Services Installed");
    }
}

fn rm_service(base_url: &str, client: reqwest::blocking::Client, args: &[String]) {
    if args.len() != 1 {
        println!("USAGE:");
        println!("riotctl rm [service]");
        return;
    }

    let endpoint = String::from(base_url) + "/services/" + &args[1];

    let res = client
        .delete(endpoint)
        .send()
        .expect("Failed to delete service");

    if res.status() != reqwest::StatusCode::OK {
        // TODO
    }
}

fn service_up(base_url: &str, client: reqwest::blocking::Client, args: &[String]) {
    if args.len() < 1 {
        println!("USAGE:");
        println!("riotctl up [services...]");
        return;
    }

    for name in args {
        update_status(base_url, &client, name, "up");
    }
}

fn service_down(base_url: &str, client: reqwest::blocking::Client, args: &[String]) {
    if args.len() < 1 {
        println!("USAGE:");
        println!("riotctl down [services...]");
        return;
    }

    for name in args {
        update_status(base_url, &client, name, "down");
    }
}

fn update_status(
    base_url: &str,
    client: &reqwest::blocking::Client,
    service: &String,
    status: &'static str,
) {
    let endpoint = String::from(base_url) + "/services/" + &service + "/" + status;

    let res = match client.get(endpoint).send() {
        Ok(x) => x,
        Err(err) => {
            println!("Failed to update status {:?}", err);
            return;
        }
    };

    if res.status() != reqwest::StatusCode::OK {
        // TODO
        println!(
            "Failed to update status, got code {}: {}",
            res.status(),
            res.text().unwrap_or_default()
        );
    }
}
