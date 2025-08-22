mod h3_server;

use axum::{
    extract::Query,
    response::Json,
    routing::get,
    Router,
};
use h3_server::H3Server;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::fmt;

#[derive(Serialize, Deserialize)]
struct HelloResponse {
    message: String,
}

#[derive(Deserialize)]
struct HelloQuery {
    name: Option<String>,
}

async fn hello_handler(Query(params): Query<HelloQuery>) -> Json<HelloResponse> {
    let name = params.name.unwrap_or_else(|| "World".to_string());
    Json(HelloResponse {
        message: format!("Hello, {}!", name),
    })
}

async fn health_handler() -> Json<HashMap<&'static str, &'static str>> {
    let mut response = HashMap::new();
    response.insert("status", "healthy");
    response.insert("version", "1.0.0");
    Json(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt::init();

    let app = Router::new()
        .route("/", get(hello_handler))
        .route("/health", get(health_handler))
        .layer(CorsLayer::permissive());

    let http_addr = SocketAddr::from(([127, 0, 0, 1], 4433));
    let h3_addr = SocketAddr::from(([127, 0, 0, 1], 4433));

    info!("Starting dual-protocol server:");
    info!("  HTTP/1.1 server: http://localhost:3000");
    info!("  HTTP/3 server: https://localhost:4433 (requires HTTP/3 client)");
    info!("Test endpoints: /, /health, /?name=YourName");

    let http_server = async move {
        let listener = TcpListener::bind(http_addr).await?;
        axum::serve(listener, app).await
    };

    let h3_server = async move {
        let server = H3Server::new(h3_addr).await?;
        server.run().await
    };

    tokio::select! {
        result = http_server => {
            if let Err(e) = result {
                error!("HTTP/1.1 server error: {}", e);
            }
        }
        result = h3_server => {
            if let Err(e) = result {
                error!("HTTP/3 server error: {}", e);
            }
        }
    }

    Ok(())
}
