use std::env;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use axum::extract::State;
use axum::{Json, Router, extract::Request, http::StatusCode, middleware::Next, routing::get};
use chrono::{DateTime, Utc};
use riot::Riot;

const VERSION: &'static str = "v0.1.0";

#[tokio::main]
async fn main() {
    let svdir: String = match env::var("SVDIR") {
        Ok(val) => val,
        Err(_) => panic!("Set $SVDIR"),
    };

    let staging_dir: String = match env::var("RIOT_STAGING") {
        Ok(val) => val,
        Err(_) => String::from(".riot"),
    };

    let state = Arc::new(RwLock::new(Riot::new(svdir.into(), staging_dir.into())));

    let app = Router::new()
        .route("/", get(get_version))
        .route(
            "/services",
            get(get_services).post(post_service).delete(del_service),
        )
        .layer(axum::middleware::from_fn(request_logger))
        .with_state(state);

    // TODO configurable interface and port
    let binding = "0.0.0.0:3000";
    println!("Listening on {}", &binding);
    let listener = tokio::net::TcpListener::bind(binding).await.unwrap();
    let _ = axum::serve(listener, app).await;
}

async fn request_logger(request: Request, next: Next) -> axum::response::Response {
    // dbg!(&request);
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();

    println!(
        "[{}] {} {} {:?}",
        now,
        &request.method(),
        &request.uri(),
        &request.version()
    );
    next.run(request).await
}

async fn get_version(State(_): State<Arc<RwLock<Riot>>>) -> Json<&'static str> {
    Json::from(VERSION)
}

async fn get_services(State(state): State<Arc<RwLock<Riot>>>) -> (StatusCode, Json<Vec<String>>) {
    let lock = state.clone();
    let riot = lock.read().unwrap();
    let out = riot.services.clone();

    (StatusCode::OK, Json::from(out))
}

async fn post_service() {}

async fn del_service() {}
