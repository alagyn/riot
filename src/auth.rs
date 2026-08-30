use std::{
    fmt::Debug, fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, path::PathBuf, sync::Arc,
};

use axum_server::tls_rustls::RustlsConfig;
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::pki_types::pem::PemObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RiotTLSConfig {
    pub server_name: String,
    pub server_cert: String,
    pub client_key: String,
}

const RIOT_CERT_NAME: &'static str = "riot_cert.pem";
const RIOT_KEY_NAME: &'static str = "riot_key.pem";

pub async fn get_tls_config(config_dir: &PathBuf, server_name: &str) -> RustlsConfig {
    let cert_path = config_dir.join(RIOT_CERT_NAME);
    let key_path = config_dir.join(RIOT_KEY_NAME);

    if !cert_path.exists() && !key_path.exists() {
        let CertifiedKey { cert, signing_key } =
            generate_simple_self_signed(vec![server_name.to_string()]).unwrap();
        let mut cert_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o611)
            .open(&cert_path)
            .unwrap();

        cert_file.write(&cert.pem().into_bytes()).unwrap();

        let mut key_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&key_path)
            .unwrap();

        key_file
            .write(&signing_key.serialize_pem().into_bytes())
            .unwrap();

        println!(
            "-- Generated new self-signed certificate, \
                call 'riotd --make-client riot.json' to generate a client config  --"
        );
    } else if !cert_path.exists() || !key_path.exists() {
        panic!(
            "Riot self-signed certificate missing, please delete {} and regenerate configs",
            config_dir.to_str().unwrap()
        );
    }

    let server_cert = rustls::pki_types::CertificateDer::from_pem_file(cert_path).unwrap();
    let server_cert_list: Vec<_> = vec![server_cert.clone()];
    let mut roots = rustls::RootCertStore::empty();

    let server_key = rustls::pki_types::PrivateKeyDer::from_pem_file(key_path).unwrap();

    roots.add(server_cert).unwrap();

    let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .unwrap();

    let config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(server_cert_list, server_key)
        .unwrap();

    RustlsConfig::from_config(Arc::new(config))
}

pub fn load_riotctl_config(config_file: &PathBuf) -> Result<RiotTLSConfig, String> {
    if !config_file.is_file() {
        return Err(format!("{} is not a file", config_file.to_str().unwrap()));
    }

    let contents = match std::fs::read(config_file) {
        Ok(x) => x,
        Err(msg) => return Err(format!("Error reading file {}", msg.to_string())),
    };

    match serde_json::from_slice::<RiotTLSConfig>(&contents) {
        Ok(x) => Ok(x),
        Err(msg) => Err(msg.to_string()),
    }
}

pub async fn gnerate_client_config(
    config_dir: &PathBuf,
    server_name: &str,
    server_port: &u16,
    output: PathBuf,
) {
    let connect_name: String;
    if *server_port == 443 {
        connect_name = server_name.to_string();
    } else {
        connect_name = server_name.to_string() + ":" + &server_port.to_string();
    }

    let server_cert = config_dir.join(RIOT_CERT_NAME);
    let server_cert: String = std::fs::read_to_string(server_cert).unwrap();

    let server_key = config_dir.join(RIOT_KEY_NAME);
    // Read in contents
    let server_key: String = std::fs::read_to_string(server_key).unwrap();
    // Pull into a KeyPair
    let server_key = rcgen::KeyPair::from_pem(&server_key).unwrap();

    let issuer = rcgen::Issuer::from_ca_cert_pem(server_cert.as_str(), server_key).unwrap();

    let client_key = rcgen::KeyPair::generate().unwrap();

    let client_cert = rcgen::CertificateParams::new(vec![server_name.to_string() + "-client"])
        .unwrap()
        .signed_by(&client_key, &issuer)
        .unwrap();

    let config_data = RiotTLSConfig {
        server_name: connect_name,
        server_cert: server_cert,
        client_key: client_key.serialize_pem() + &client_cert.pem(),
    };

    let mut config_file = OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o600)
        .truncate(true)
        .open(&output)
        .unwrap();

    config_file
        .write(&serde_json::to_string(&config_data).unwrap().into_bytes())
        .unwrap();
}
