use clap::Parser;
use openssl::pkey::PKey;
use reqwest::blocking::Client;
use rustls::{Certificate, ServerConfig, ServerConnection, StreamOwned};
use rustls_pemfile::Item;
use std::error::Error;
use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use std::net::SocketAddr;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

fn load_tls_config() -> Arc<ServerConfig> {
    let private_key_bytes = include_bytes!("../../self-signed-certs/server.pem");
    let pkey = PKey::private_key_from_pem(private_key_bytes).expect("Failed to parse private key");

    let private_key = rustls::PrivateKey(
        pkey.private_key_to_der()
            .expect("Failed to encode private key"),
    );

    let certs = load_certs("self-signed-certs/server.crt");

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs.unwrap(), private_key)
        .expect("bad certificate/key");

    Arc::new(config)
}

fn load_certs(path: &str) -> Result<Vec<Certificate>, io::Error> {
    let cert_file = File::open(path)?;
    let mut cert_reader = BufReader::new(cert_file);

    let mut certs = Vec::new();
    while let Some(item) = rustls_pemfile::read_one(&mut cert_reader)? {
        if let Item::X509Certificate(cert) = item {
            certs.push(Certificate(cert.to_vec()));
        }
    }

    Ok(certs)
}

fn handle_connection(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
) -> Result<(), Box<dyn Error>> {
    let server_conn = ServerConnection::new(tls_config).unwrap();
    let mut tls_stream = StreamOwned::new(server_conn, stream);

    let buf_reader = BufReader::new(&mut tls_stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    let client = Client::new();

    let response = client.get("https://www.google.com/").send()?;

    if response.status().is_success() {
        let body = response.text()?;

        println!("Response: {}", body);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        tls_stream.write_all(response.as_bytes())?;
    } else {
        let error_response = format!(
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 19\r\n\r\nGoogle Request Failed"
        );
        tls_stream.write_all(error_response.as_bytes())?;
        println!("Failed to fetch response. Status: {}", response.status());
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));

    let listener = TcpListener::bind(addr).unwrap();

    let tls_config = load_tls_config();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let _ = handle_connection(stream, tls_config.clone());
    }
}
