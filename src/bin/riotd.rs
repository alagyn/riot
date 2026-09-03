use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router, extract::Request, http::StatusCode, middleware::Next, routing::get};

use axum_server::tls_rustls::RustlsAcceptor;
use axum_server_mtls::MtlsAcceptor;
use chrono::{DateTime, Local};
use riot;
use riot::Riot;
use riot::auth::get_tls_config;
use riot::responses::{ServiceList, ServiceStatusRes};

type RiotState = State<Arc<RwLock<Riot>>>;

#[tokio::main]
async fn main() {
    // Load configs
    let svdir: String = match env::var("SVDIR") {
        Ok(val) => val,
        Err(_) => panic!("Set $SVDIR"),
    };

    let staging_dir: String = match env::var("RIOT_STAGING") {
        Ok(val) => val,
        Err(_) => String::from(".riot"),
    };

    let server_name = match env::var("RIOT_SERVERNAME") {
        Ok(val) => val,
        Err(_) => String::from("localhost"),
    };

    let server_port: u16 = match env::var("RIOT_SERVERPORT") {
        Ok(val) => val.parse().unwrap(),
        Err(_) => 443,
    };

    // Init state
    let riot = match Riot::new(svdir.into(), staging_dir.into()) {
        Ok(x) => x,
        Err(msg) => panic!("{}", msg),
    };

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if args[1] == "--make-client" {
            if args.len() != 3 {
                println!("Usage: riotd --make-client [output.json]");
                return;
            }

            let output = args[2].clone();
            riot::auth::gnerate_client_config(
                &riot.config_dir,
                &server_name,
                &server_port,
                PathBuf::from(&output),
            )
            .await;
            println!("Generated new client config {}", output);
            return;
        }
    }

    let tls_cfg = get_tls_config(&riot.config_dir, &server_name).await;
    let state = Arc::new(RwLock::new(riot));

    // Init router
    let app = Router::new()
        .route("/", get(get_version))
        .route("/services", get(get_services))
        .route(
            "/services/{service_name}",
            post(post_service).delete(del_service),
        )
        .route("/services/{service_name}/{status}", get(manage_service))
        // .layer(axum::middleware::from_fn(auth_verify))
        .layer(axum::middleware::from_fn(request_logger))
        .with_state(state);

    // TODO configurable interface and port
    let addr = SocketAddr::from(([0, 0, 0, 0], server_port));
    let acceptor = MtlsAcceptor::new(RustlsAcceptor::new(tls_cfg));

    println!("Listening on {}", &addr);
    axum_server::bind(addr)
        .acceptor(acceptor)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn request_logger(request: Request, next: Next) -> axum::response::Response {
    // dbg!(&request);
    let now = SystemTime::now();
    let now: DateTime<Local> = now.into();
    let now = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let log = format!(
        "[{}] {} {} {:?}",
        now,
        &request.method(),
        &request.uri(),
        &request.version()
    );
    let res = next.run(request).await;
    println!("{} -> ({})", log, res.status());
    res
}

async fn get_version(State(_): RiotState) -> Json<&'static str> {
    Json::from(riot::VERSION)
}

async fn get_services(State(state): RiotState) -> (StatusCode, Json<ServiceList>) {
    let lock = state.clone();
    let riot = lock.read().unwrap();
    let out = riot.list_services();
    (StatusCode::OK, Json::from(out))
}

async fn post_service(
    State(state): RiotState,
    Path(service_name): Path<String>,
    body: Bytes,
) -> (StatusCode, String) {
    println!("Service posted {}", service_name);
    let lock = state.clone();
    let mut riot = lock.write().unwrap();

    // TODO check if we are already tracking service and it is enabled
    // if so, we should restart the service

    // TODO check if there is a service with the same name already
    // in the svdir, but not one we are tracking. Should Error

    let install_dir = riot.install_dir.clone().join(&service_name);
    if !install_dir.is_dir() {
        if install_dir.is_file() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Cannot create service direcotry, a file with the same name exists"),
            );
        }

        match std::fs::create_dir(&install_dir) {
            Ok(_) => (),
            Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
        }
    }

    let run_script = install_dir.clone().join("run");

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o744)
        .open(run_script);
    let mut file = match file {
        Ok(x) => x,
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
    };

    match file.write_all(body.iter().as_slice()) {
        Ok(_) => (),
        Err(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
    };

    riot.services.push(riot::service::Service {
        name: service_name,
        enabled: false,
    });
    (StatusCode::CREATED, String::from(""))
}

async fn del_service() {
    // TODO
    // Delete a service, should probably warn users if there are any files left over in the
    // staging folder
    panic!("unimplemented")
}

async fn manage_service(
    State(state): RiotState,
    Path((service_name, status)): Path<(String, String)>,
) -> (StatusCode, Json<Option<ServiceStatusRes>>) {
    println!(
        "Trying to update service {} to state {}",
        service_name, status
    );
    let lock = state.clone();
    let riot = lock.write().unwrap();

    let service = match riot.services.iter().find(|x| x.name == service_name) {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, Json::from(None)),
    };

    match status.as_str() {
        "up" => riot.start_service(&service),
        "down" => riot.stop_service(&service),
        _ => return (StatusCode::BAD_REQUEST, Json::from(None)),
    }

    (
        StatusCode::OK,
        Json::from(Some(riot.check_service_status(&service))),
    )
}
