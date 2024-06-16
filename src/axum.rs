//! Run with
//!
//! ```not_rust
//! $ cargo run -p example-http-proxy
//! ```
//!
//! In another terminal:
//!
//! ```not_rust
//! $ curl -v -x "127.0.0.1:3000" https://tokio.rs
//! ```
//!
//! Example is based on <https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs>

use axum::{
    body::{box_body, Body},
    http::{Method, Request, Response, StatusCode},
    routing::get,
    Router,
  };
  use futures_util::TryFutureExt;
  use hyper::client::connect::dns::GaiResolver;
  use hyper::client::HttpConnector;
  use hyper::service::{make_service_fn, service_fn};
  use hyper::upgrade::Upgraded;
  use hyper::{Client, Error, Server};
  use hyper_tls::HttpsConnector;
  use std::net::SocketAddr;
  use tokio::net::TcpStream;
  use tower::{make::Shared, ServiceExt};
  
  // #[cfg(feature = "https")]
  fn build_client() -> Client<hyper_tls::HttpsConnector<HttpConnector<GaiResolver>>, hyper::Body> {
    // 4 is number of blocking DNS threads
    let https = HttpsConnector::new();
    Client::builder().build::<_, hyper::Body>(https)
  }
  
  // #[cfg(not(feature = "https"))]
  // fn build_client() -> Client<HttpConnector<GaiResolver>, hyper::Body> {
  //     Client::new()
  // }
  
  #[tokio::main]
  async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "rust_proxy_tokio=trace,tower_http=debug")
    }
    tracing_subscriber::fmt::init();
    tracing::info!("preparing to shave yaks");
  
    let service = tower::service_fn(move |mut req: Request<Body>| {
        println!("{:?}", req);
        let client = build_client();
        let host_addr = "https://echo.hoppscotch.io";
        async move {
            let uri_string = format!(
                "https://{}{}",
                &host_addr,
                req.uri()
                    .path_and_query()
                    .map(|x| x.as_str())
                    .unwrap_or("/")
            );
            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;
            let resp = client.request(req).map_ok(|res| {
                println!("{:?}", res);
                res
            });
            println!("{:?}", resp.copy().await);
            resp.await
        }
    });
  
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(Shared::new(service))
        .await
        .unwrap();
  }
  
  // async fn proxy(
  //     req: Request<Body>,
  //     client: Client<HttpsConnector<HttpConnector<GaiResolver>>, Body>,
  // ) -> Result<Response<Body>, hyper::Error> {
  //     tracing::trace!(?req);
  //     let host_addr = "https://tokio.rs";
  //     // if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
  //     println!("{:?}", host_addr);
  //     tokio::task::spawn(async move {
  //         match hyper::upgrade::on(req).await {
  //             Ok(upgraded) => {
  //                 if let Err(e) = tunnel(upgraded, host_addr.to_string()).await {
  //                     tracing::warn!("server io error: {}", e);
  //                 };
  //             }
  //             Err(e) => tracing::warn!("upgrade error: {}", e),
  //         }
  //     });
  
  //     Ok(Response::new(Body::empty()))
  //     // } else {
  //     //     tracing::warn!("CONNECT host is not socket addr: {:?}", req.uri());
  //     //     let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
  //     //     *resp.status_mut() = StatusCode::BAD_REQUEST;
  
  //     //     Ok(resp)
  //     // }
  // }