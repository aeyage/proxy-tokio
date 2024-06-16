// Import necessary libraries and modules
use axum::{
    http::{Request, Response},
    response::IntoResponse,
    routing::get,
    AddExtensionLayer, Router,
};
use hyper::Body;
use hyper::StatusCode;

use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use proxy_tokio::{build_client, call, HyperClient};

// Proxy function to handle incoming requests and forwards them to the target server
async fn proxy(req: Request<hyper::Body>) -> impl IntoResponse {
    let client_ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0)
        .unwrap();
    let client = req.extensions().get::<HyperClient>().unwrap().clone();

    match call(client_ip, "https://echo.hoppscotch.io", client, req).await {
        Ok(response) => response,
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap(),
    }
}

// Main function to set up the server and its middleware
#[tokio::main]
async fn main() {
    // Set the RUST_LOG if it has not been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "rust_proxy_tokio=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    let client = build_client();
    // If you want to customise the behavior using closures here is how you can do it

    let middleware_stack = ServiceBuilder::new()
        // Handle errors from middleware
        //
        // This middleware most be added above any fallible
        // ones if you're using `ServiceBuilder`, due to how ordering works
        // .layer(HandleErrorLayer::new(handle_error))
        // `TraceLayer` adds high level tracing and logging
        .layer(TraceLayer::new_for_http())
        // .layer(TimeoutLayer::new(Duration::from_secs(5)))
        // .layer(ConcurrencyLimitLayer::new(1))
        // `AsyncFilterLayer` lets you asynchronously transform the request
        // .layer(AsyncFilterLayer::new(map_request))
        // `AndThenLayer` lets you asynchronously transform the response
        // .layer(AndThenLayer::new(map_response))
        .layer(AddExtensionLayer::new(client));

    // Build our application with a route
    let app = Router::new()
        .route("/*path", get(proxy).post(proxy))
        .layer(middleware_stack);

    // let service = middleware_stack()

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    
    axum::Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
        .await
        .unwrap();
}