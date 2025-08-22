use bytes::Bytes;
use h3::{error::ErrorLevel, quic::BidiStream, server::RequestStream};
use h3_quinn::quinn;
use http::{Method, Request, Response, StatusCode};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tracing::{error, info, warn};

pub struct H3Server {
    endpoint: quinn::Endpoint,
    addr: SocketAddr,
}

impl H3Server {
    pub async fn new(addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let (cert, key) = generate_self_signed_cert()?;
        
        let mut server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)?;

        server_config.alpn_protocols = vec![b"h3".to_vec()];

        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_bidi_streams(100u32.into());
        transport_config.max_concurrent_uni_streams(100u32.into());
        transport_config.max_idle_timeout(Some(Duration::from_secs(30).try_into().unwrap()));
        transport_config.keep_alive_interval(Some(Duration::from_secs(10)));

        let server_config = quinn::ServerConfig::with_crypto(Arc::new(server_config));
        let endpoint = quinn::Endpoint::server(server_config, addr)?;

        Ok(Self { endpoint, addr })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("HTTP/3 server listening on {}", self.addr);

        while let Some(conn) = self.endpoint.accept().await {
            let conn = conn.await?;
            info!(
                "New HTTP/3 connection established: {}",
                conn.remote_address()
            );

            tokio::spawn(async move {
                if let Err(err) = handle_connection(conn).await {
                    error!("HTTP/3 connection error: {}", err);
                }
            });
        }

        Ok(())
    }
}

async fn handle_connection(
    conn: quinn::Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn)).await?;

    loop {
        match h3_conn.accept().await {
            Ok(Some((req, stream))) => {
                tokio::spawn(async move {
                    if let Err(err) = handle_request(req, stream).await {
                        error!("Request handling error: {}", err);
                    }
                });
            }
            Ok(None) => break,
            Err(err) => {
                match err.get_error_level() {
                    ErrorLevel::ConnectionError => {
                        error!("Connection error: {}", err);
                        break;
                    }
                    ErrorLevel::StreamError => {
                        warn!("Stream error: {}", err);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_request<T>(
    req: Request<()>,
    mut stream: RequestStream<T, Bytes>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: BidiStream<Bytes>,
{
    info!("Received HTTP/3 request: {} {}", req.method(), req.uri().path());

    let (response, body_data) = match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let name = extract_name_from_query(req.uri().query());
            let body = serde_json::json!({
                "message": format!("Hello, {} via HTTP/3!", name),
                "protocol": "HTTP/3"
            });
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(())?;
            (response, Some(Bytes::from(body.to_string())))
        }
        (&Method::GET, "/health") => {
            let body = serde_json::json!({
                "status": "healthy",
                "version": "1.0.0",
                "protocol": "HTTP/3"
            });
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(())?;
            (response, Some(Bytes::from(body.to_string())))
        }
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(())?;
            (response, Some(Bytes::from("Not Found")))
        }
    };

    stream.send_response(response).await?;
    
    if let Some(data) = body_data {
        stream.send_data(data).await?;
    }
    
    stream.finish().await?;

    Ok(())
}

fn extract_name_from_query(query: Option<&str>) -> String {
    query
        .and_then(|q| {
            q.split('&')
                .find_map(|param| {
                    let mut parts = param.splitn(2, '=');
                    if parts.next()? == "name" {
                        parts.next()
                    } else {
                        None
                    }
                })
        })
        .unwrap_or("World")
        .to_string()
}

fn generate_self_signed_cert() -> Result<(Certificate, PrivateKey), Box<dyn std::error::Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_der = cert.serialize_der()?;
    let private_key_der = cert.serialize_private_key_der();

    Ok((
        Certificate(cert_der),
        PrivateKey(private_key_der),
    ))
}

