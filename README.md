## Rust Proxy Server using `Tokio.rs`, `Hyper` & `Axum`

>HTTP proxy server using the Tokio.rs stack along with the Hyper and Axum crates. It involves handling incoming HTTP requests and processing them such as tracing, authorizing, logging, measuring, and monitoring, and then proxying them to a target server.
  
## Getting started

```bash
cargo run
```

Once the server is up and running, send a test request to it using the curl command below:

```bash
curl -X POST \
  '127.0.0.1:3000' \
  -H 'content-type: application/json' \
  -H 'Content-Type: application/json; charset=utf-8' \
  -d '{"test":"test"}'
```

The command sends a POST request to the local server, which then proxies the request to the specified endpoint `https://echo.hoppscotch.io/` and returns the response.

## Resources

- [`Tokio`](https://tokio.rs)
- [Hyper http proxy](https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs)
- [hyper proxy crate](https://crates.io/crates/hyper-proxy)
- [medium blog post](https://medium.com/swlh/writing-a-proxy-in-rust-and-why-it-is-the-language-of-the-future-265d8bf7c6d2)
- [hyper reverse proxy crate](https://github.com/felipenoris/hyper-reverse-proxy)
- [`axum` example](https://github.com/tokio-rs/axum/blob/main/examples/http-proxy/src/main.rs)
- [`actix-web` http proxy](https://github.com/actix/examples/blob/master/basics/http-proxy/src/main.rs)
- [`axum` tls example](https://github.com/tokio-rs/axum/blob/main/examples/tls-rustls/src/main.rs)
