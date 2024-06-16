use hyper::client::connect::dns::GaiResolver;
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error, Server};
use hyper_tls::HttpsConnector;

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
    pretty_env_logger::init();

    let in_addr = ([127, 0, 0, 1], 3001).into();
    let out_addr = "echo.hoppscotch.io";

    let client_main = build_client();

    // The closure inside `make_service_fn` is run for each connection,
    // creating a 'service' to handle requests for that specific connection.
    let make_service = make_service_fn(move |_| {
        let client = client_main.clone();

        async move {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, Error>(service_fn(move |mut req| {
                let uri_string = format!(
                    "https://{}{}",
                    &out_addr,
                    req.uri()
                        .path_and_query()
                        .map(|x| x.as_str())
                        .unwrap_or("/")
                );
                let uri = uri_string.parse().unwrap();
                *req.uri_mut() = uri;
                client.request(req)
            }))
        }
    });

    let server = Server::bind(&in_addr).serve(make_service);

    println!("Listening on http://{}", in_addr);
    println!("Proxying on http://{}", out_addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}