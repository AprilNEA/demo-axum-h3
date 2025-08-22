# Demo Axum HTTP/3 Server

This project demonstrates an Axum-based HTTP/1.1 API server with plans for HTTP/3 support.

## Current Features

- Axum HTTP/1.1 server on port 3000
- JSON API endpoints:
  - `GET /` - Hello endpoint with optional `name` query parameter
  - `GET /health` - Health check endpoint
- CORS support
- Structured logging with tracing

## API Endpoints

### Hello Endpoint
```bash
curl "http://localhost:3000"
curl "http://localhost:3000?name=World"
```

Response:
```json
{
  "message": "Hello, World!"
}
```

### Health Check
```bash
curl "http://localhost:3000/health"
```

Response:
```json
{
  "status": "healthy",
  "version": "1.0.0"
}
```

## Running the Server

```bash
cargo run
```

The server will start and display:
- HTTP/1.1 server listening on 127.0.0.1:3000
- Visit http://localhost:3000 or http://localhost:3000?name=YourName
- Health check: http://localhost:3000/health

## HTTP/3 Implementation Notes

HTTP/3 implementation using the Quinn QUIC library and H3 crate requires:
- Complex version compatibility between quinn, h3, and h3-quinn crates
- Self-signed certificate generation for QUIC/TLS
- UDP socket configuration for QUIC transport

The current implementation focuses on a working HTTP/1.1 Axum server as the foundation. HTTP/3 can be added when the ecosystem stabilizes with better version compatibility.

## Dependencies

- `axum` - Web application framework
- `tokio` - Async runtime
- `tower` & `tower-http` - Middleware layers
- `serde` & `serde_json` - JSON serialization
- `tracing` - Structured logging