use std::{env, path::PathBuf};

use riot;

type CmdFunc = fn(&str, &[String]) -> ();

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

    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        usage(&cmds);
        return;
    }

    if args[1] == "help" {
        usage(&cmds);
        return;
    }

    // TODO load from config
    let base_url = "http://localhost:3000";

    for cmd in &cmds {
        if cmd.name == args[1] {
            (cmd.func)(base_url, &args[2..]);
            return;
        }
    }

    usage(&cmds);
}

fn send_service(base_url: &str, args: &[String]) {
    if args.len() != 1 {
        println!("USAGE:");
        println!("riotctl push [service.sh]");
        return;
    }

    let service_file: PathBuf = args[1].clone().into();
    if !service_file.is_file() {
        println!("{} is not a file", &service_file.to_str().unwrap());
        return;
    }

    let body = std::fs::read(service_file).expect("Failed to read service file");
    let endpoint = String::from(base_url) + "/services";

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(endpoint)
        .body(body)
        .send()
        .expect("Failed to push service");

    if res.status() != reqwest::StatusCode::OK {
        panic!("Failed to push service {}", res.text().unwrap());
    }

    // TODO
    println!("Created");
}

fn list_services(base_url: &str, _args: &[String]) {
    let endpoint = String::from(base_url) + "/services";

    let res = reqwest::blocking::get(endpoint).expect("Failed to list services");
    if res.status() != reqwest::StatusCode::OK {
        // TODO?
        println!("Failed to list services");
        return;
    }

    // TODO this should probably print out the statuses?
    let services: Vec<String> = res.json().unwrap();
    if !services.is_empty() {
        println!("Services:");
        for x in services {
            println!("{}", x);
        }
    } else {
        println!("No Services Installed");
    }
}

fn rm_service(base_url: &str, args: &[String]) {
    if args.len() != 1 {
        println!("USAGE:");
        println!("riotctl rm [service]");
    }

    let endpoint = String::from(base_url) + "/services/" + &args[1];

    let client = reqwest::blocking::Client::new();
    let res = client
        .delete(endpoint)
        .send()
        .expect("Failed to delete service");

    if res.status() != reqwest::StatusCode::OK {}
}
