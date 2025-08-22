# Demo Axum HTTP/3 Server

This project demonstrates an Axum-based API server with HTTP/3 support.

## Running the Server

```bash
cargo run
```

## Current Features
- Axum HTTP/1.1 and HTTP/3 server on port 4433

### Testing HTTP/1.1
```bash
curl "http://localhost:3000"
```

### Testing HTTP/2
```bash
curl --http2 "http://localhost:4433"
# Only use HTTP/2, fail if the server does not support it
curl --http2-prior-knowledge http://localhost:4433
```


### Testing HTTP/3
```bash
curl --http3-only -k "https://localhost:4433"
```

> Make sure to install `curl` with HTTP/3 support:
>
> ```bash
> brew remove -f curl
> brew install cloudflare/homebrew-cloudflare/curl
> echo 'export PATH="/usr/local/opt/curl/bin:$PATH"' >> ~/.zshrc
> source ~/.zshrc
> ```
> 
> Credit: [Installing curl with http3 on MacOS](https://gist.github.com/xmlking/cff9510dac9281d29390392cbbb033a8)


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