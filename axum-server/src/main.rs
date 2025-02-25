//! Run with
//!
//! ```not_rust
//! cargo run -p example-tls-rustls
//! ```

#![allow(unused_imports)]

use axum::{
    handler::HandlerWithoutStateExt,
    http::{uri::Authority, StatusCode, Uri, Method, HeaderValue, HeaderName},
    response::Redirect,
    routing::get,
    BoxError, Router, Json
};
use tower_http::cors::CorsLayer;
use tower::util::ServiceExt;
use axum_extra::extract::Host;
use axum_server::tls_rustls::RustlsConfig;
use std::{net::SocketAddr, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use serde_json::json;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use axum::extract::Request;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // tokio::join!(
    //     serve(using_serve_dir(), 3001),
    //     serve(using_serve_dir_with_assets_fallback(), 3002),
    //     serve(using_serve_dir_only_from_root_via_fallback(), 3003),
    //     serve(using_serve_dir_with_handler_as_service(), 3004),
    //     serve(two_serve_dirs(), 3005),
    //     serve(calling_serve_dir_from_a_handler(), 3006),
    //     serve(using_serve_file_from_a_route(), 3307),
    // );

    // optional: spawn a second server to redirect http requests to this server

    // let config = RustlsConfig::from_pem_file(
    //     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .join("self_signed_certs")
    //         .join("cert.pem"),
    //     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .join("self_signed_certs")
    //         .join("key.pem"),
    // )
    // .await
    // .unwrap();

    let cert = env::var("TLS_CRT")
    .expect("TLS_CRT must be set");

    println!("Cert length: {:?}", cert);

    let key = env::var("TLS_KEY")
        .expect("TLS_KEY must be set");
    println!("Key length: {}", key.len());

    let config = RustlsConfig::from_pem(
            cert.as_bytes().to_vec(),  
            key.as_bytes().to_vec(),   
        )
        .await
        .expect("Failed to load TLS config");

    let serve_dir = ServeDir::new("assets")
        .not_found_service(ServeFile::new("assets/index.html"));

    let app = Router::new()
        .route("/wtransport", get(handler))
        .nest_service("/assets", serve_dir.clone())
        .fallback_service(serve_dir);

    // run https server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[allow(dead_code)]
async fn handler() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, World!" }))
}

fn using_serve_dir() -> Router {
    // serve the file in the "assets" directory under `/assets`
    Router::new().nest_service("/assets", ServeDir::new("assets"))
}

fn using_serve_dir_with_assets_fallback() -> Router {
    // `ServeDir` allows setting a fallback if an asset is not found
    // so with this `GET /assets/doesnt-exist.jpg` will return `index.html`
    // rather than a 404
    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .nest_service("/assets", serve_dir.clone())
        .fallback_service(serve_dir)
}

fn using_serve_dir_only_from_root_via_fallback() -> Router {
    // you can also serve the assets directly from the root (not nested under `/assets`)
    // by only setting a `ServeDir` as the fallback
    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .fallback_service(serve_dir)
}

fn using_serve_dir_with_handler_as_service() -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }

    // you can convert handler function to service
    let service = handle_404.into_service();

    let serve_dir = ServeDir::new("assets").not_found_service(service);

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .fallback_service(serve_dir)
}

fn two_serve_dirs() -> Router {
    // you can also have two `ServeDir`s nested at different paths
    let serve_dir_from_assets = ServeDir::new("assets");
    let serve_dir_from_dist = ServeDir::new("dist");

    Router::new()
        .nest_service("/assets", serve_dir_from_assets)
        .nest_service("/dist", serve_dir_from_dist)
}

#[allow(clippy::let_and_return)]
fn calling_serve_dir_from_a_handler() -> Router {
    // via `tower::Service::call`, or more conveniently `tower::ServiceExt::oneshot` you can
    // call `ServeDir` yourself from a handler
    Router::new().nest_service(
        "/foo",
        get(|request: Request| async {
            let service = ServeDir::new("assets");
            let result = service.oneshot(request).await;
            result
        }),
    )
}

fn using_serve_file_from_a_route() -> Router {
    Router::new().route_service("/foo", ServeFile::new("assets/index.html"))
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}
