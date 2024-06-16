use axum::{
    body::{BoxBody, Bytes},
    error_handling::HandleErrorLayer,
    http::{HeaderMap, Request, Response},
    response::{Html, IntoResponse},
    routing::get,
    AddExtensionLayer, Router,
};
use futures_util::TryFutureExt;
use hyper::{
    client::{connect::dns::GaiResolver, HttpConnector},
    header::{FORWARDED, HOST},
    Body, Client,
};
use hyper_tls::HttpsConnector;
use std::{convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tower::{
    filter::AsyncFilterLayer, limit::ConcurrencyLimitLayer, timeout::TimeoutLayer,
    util::AndThenLayer, ServiceBuilder,
};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;

#[derive(Clone)]
struct State {
    counter: u8,
}

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

async fn map_request(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    // println!("{:?}", req);
    // let parts = req.uri().path();
    // println!("{:?}", parts);
    // println!("request");
    Ok(req)
}

async fn map_response(res: Response<BoxBody>) -> Result<Response<BoxBody>, Infallible> {
    // println!("{:?}", res);
    // println!("response");
    Ok(res)
}

fn handle_error<T>(error: T) -> axum::http::StatusCode {
    axum::http::StatusCode::INTERNAL_SERVER_ERROR
}

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "rust_proxy_tokio=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    let client = build_client();
    // If you want to customize the behavior using closures here is how
    //
    let middleware_stack = ServiceBuilder::new()
        // Handle errors from middleware
        //
        // This middleware most be added above any fallible
        // ones if you're using `ServiceBuilder`, due to how ordering works
        .layer(HandleErrorLayer::new(handle_error))
        // `TraceLayer` adds high level tracing and logging
        .layer(TraceLayer::new_for_http())
        // .layer(TimeoutLayer::new(Duration::from_secs(5)))
        // .layer(ConcurrencyLimitLayer::new(1))
        // `AsyncFilterLayer` lets you asynchronously transform the request
        .layer(AsyncFilterLayer::new(map_request))
        // `AndThenLayer` lets you asynchronously transform the response
        .layer(AndThenLayer::new(map_response))
        .layer(AddExtensionLayer::new(client));

    // build our application with a route
    let app = Router::new()
        .route("/*path", get(proxy).post(proxy))
        .layer(middleware_stack);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn proxy(mut req: Request<hyper::Body>) -> impl IntoResponse {
    println!("{:?}", req.headers());
    let remote_addr = req
    .extensions()
    .get::<axum::extract::ConnectInfo<SocketAddr>>()
    .map(|ci| ci.0);
    let client = req
        .extensions()
        .get::<Client<HttpsConnector<HttpConnector<GaiResolver>>>>()
        .unwrap()
        .clone();

    let host_addr = "echo.hoppscotch.io";

    let uri_string = format!(
        "https://{}{}",
        &host_addr,
        req.uri()
            .path_and_query()
            .map(|x| x.as_str())
            .unwrap_or("/")
    );
    println!("{}", uri_string);
    *req.uri_mut() = uri_string.parse().unwrap();
    // add x-forwarded-for header for host
    // req.headers_mut().insert(
    //     FORWARDED,
    //     HeaderMap::from_iter(vec![(
    //         "x-forwarded-for".parse().unwrap(),
    //         HeaderValue::from_str(&host_addr).unwrap(),
    //     )]),
    // );

    // rewrite host header
    if req.headers().get(HOST).is_some() {
        // req.headers_mut().remove(HOST).unwrap();
        req.headers_mut()
            .insert(HOST, host_addr.parse().unwrap())
            .unwrap();
    }

    // let r = req;
    println!("{:?}", req);

    let resp = client
        .request(req)
        .map_ok(futures_util::future::ready)
        .await;
    println!("{:?}", resp);

    Html("<h1>Hello, World!</h1>")
}